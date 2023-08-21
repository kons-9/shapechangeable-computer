use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_hal::{
    gpio::{AnyOutputPin, PinDriver},
    prelude::*,
};
use network_node::serial::SerialTrait;
use network_node::{flit::Flit, header::Header};
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
    display.print("start packet test...", true);

    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let hertz = 115200;

    let mut serial = Serial::new(uart, tx, rx, enable, hertz);

    let ip_address = Efuse::new().get_efuse_value();

    let header = Header::HCheckConnection;
    let flit = Flit::make_head_flit(0, header, ip_address, 0xFF, 0);

    loop {
        match flit.send(&mut serial, header.is_require_ack()) {
            Ok(_) => {
                display.print("send success", true);
            }
            Err(e) => {
                display.print(&format!("send failed: {:?}", e), true);
                continue;
            }
        };

        loop {
            match Flit::receive(&mut serial) {
                Ok(data) => {
                    if let Some(data) = data {
                        if let Ok((len, head, src, dst, pid)) = Flit::get_head_information(data) {
                            if src == ip_address {
                                display.print("receive self packet", true);
                                continue;
                            }
                            display.print("receive success", true);
                            display.print(&format!("len: {}, ", len), false);
                            display.print(&format!("head: {:?}, ", head), false);
                            display.print(&format!("src: {}, ", src), false);
                            display.print(&format!("dst: {}, ", dst), false);
                            display.print(&format!("pid: {}", pid), true);
                        }
                    } else {
                        display.print("no data", true);
                        break;
                    }
                }
                Err(e) => {
                    display.print(format!("receive failed: {:?}", e).as_str(), true);
                    serial.flush_all()?;
                    break;
                }
            };
        }

        thread::sleep(Duration::from_millis(1000));
    }
}
