use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::prelude::Peripherals;
use network_node::header::Header::HCheckConnection;
use network_node::packet::{Packet, ToId};
use std_display::efuse::Efuse;
use std_display::serial::Serial;

fn main() -> Result<()> {
    let peripheral = Peripherals::take().expect("never fails");

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
                println!("check connection");
            }
            Err(e) => {
                println!("SendError: {:?}", e);
            }
        }

        thread::sleep(Duration::from_secs(1));

        loop {
            let data = match Packet::receive(&mut serial) {
                Ok(data) => data,
                Err(e) => {
                    println!("ReceiveError: {:?}", e);
                    break;
                }
            };
            match data {
                Some(t) if t.get_global_to() == ToId::Broadcast => {
                    println!("Received: {:?}", t);
                    let header = t.get_header();
                    println!("Received: {:?}", header);
                }
                Some(t) if t.get_to() == ToId::Unicast(ip_address) => {
                    println!("Received: {:?}", t);
                    let header = t.get_header();
                    println!("Received: {:?}", header);
                }
                Some(_t) => {
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
