use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_hal::{
    gpio::{AnyOutputPin, PinDriver},
    prelude::*,
};
use std_display::display::Display;

use network_node::serial::SerialTrait;
use std_display::serial::Serial;

fn main() -> Result<()> {
    let peripheral = Peripherals::take().expect("never fails");

    // display
    let spi = peripheral.spi2;
    let sclk = peripheral.pins.gpio8;
    let dc = PinDriver::output(peripheral.pins.gpio4).expect("failed to set dc pin");
    let sdo = peripheral.pins.gpio10;
    let rst = PinDriver::output(peripheral.pins.gpio3).expect("failed to set rst pin");
    let hertz = 30.MHz().into();
    let mut display = Display::new(spi, sclk, sdo, dc, rst, hertz);

    display.print("display initialized", true);
    display.print("start serial test...", false);

    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let hertz = 115200;

    let mut serial = Serial::new(uart, tx, rx, enable, hertz);

    loop {
        match serial.send(b"hello!!!") {
            Ok(_) => {
                display.print(&format!("send data: 'hello!!!'({:?})", b"hello!!!"), true);
                display.print(&format!("bdata: {:?}", b"hello!!!"), true);
                println!("Sent: hello!!!");
                println!("Data: {:?}", b"hello!!!");
            }
            Err(e) => {
                display.print(&format!("send error: {:?}", e), true);
                println!("SendError: {:?}", e);
            }
        }
        loop {
            let data = match serial.receive() {
                Ok(data) => data,
                Err(e) => {
                    display.print(&format!("receive error: {:?}", e), true);
                    println!("ReceiveError: {:?}", e);
                    break;
                    // None
                }
            };
            match data {
                Some(t) => {
                    display.print(&format!("received: {:?}", t), true);
                    display.print(
                        &format!("received: {:?}", String::from_utf8(t.to_vec())),
                        true,
                    );
                    println!("Received: {:?}", t);
                    println!("Received: {:?}", String::from_utf8(t.to_vec()));
                }
                None => {
                    display.print("no data received", true);
                    println!("No data received");
                    break;
                }
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
