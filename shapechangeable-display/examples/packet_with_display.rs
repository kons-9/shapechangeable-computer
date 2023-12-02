use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_hal::{
    gpio::{AnyOutputPin, PinDriver},
    prelude::*,
};
use network_node::packet::{Packet, ToId};
use network_node::{header::Header::HCheckConnection, serial::SerialTrait};
use std_display::display::Display;
use std_display::efuse::Efuse;
use std_display::serial::Serial;

fn main() -> Result<()> {
    let peripheral = Peripherals::take().expect("never fails");

    let spi = peripheral.spi2;
    let sclk = peripheral.pins.gpio8;
    let dc = PinDriver::output(peripheral.pins.gpio4).expect("failed to set dc pin");
    let sdo = peripheral.pins.gpio10;
    let rst = PinDriver::output(peripheral.pins.gpio3).expect("failed to set rst pin");
    let hertz = 30.MHz().into();

    let mut display = Display::new(spi, sclk, sdo, dc, rst, hertz);

    display.print("display initialized", true);
    display.print("start packet test...", false);

    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let hertz = 115200;

    let mut serial = Serial::new(uart, tx, rx, enable, hertz);

    let ip_address = Efuse::new().get_efuse_value();
    let packet = Packet::new(
        0,
        HCheckConnection,
        ip_address,
        ToId::Broadcast,
        ip_address,
        ToId::Broadcast,
        vec![],
    );

    loop {
        match packet.send(&mut serial) {
            Ok(_) => {
                display.print("packet sent", true);
            }
            Err(e) => {
                display.print(format!("SendError: {:?}", e).as_str(), true);
            }
        }

        thread::sleep(Duration::from_secs(1));

        loop {
            let data = match Packet::receive(&mut serial) {
                Ok(data) => data,
                Err(e) => {
                    println!("ReceiveError: {:?}", e);
                    serial.flush_all()?;
                    break;
                }
            };
            match data {
                Some(t) if t.get_global_to() == ToId::Broadcast => {
                    display.print("packet received", true);
                    println!("Received: {:?}", t);
                    let header = t.get_header();
                    println!("Received: {:?}", header);
                }
                Some(t) if t.get_to() == ToId::Unicast(ip_address) => {
                    display.print("packet received", true);
                    println!("Received: {:?}", t);
                    let header = t.get_header();
                    println!("Received: {:?}", header);
                }
                Some(_t) => {
                    display.print("not mine", true);
                    println!("not mine");
                    break;
                }
                None => {
                    println!("Packet doesn't have any data received");
                    break;
                }
            }
        }
    }
}
