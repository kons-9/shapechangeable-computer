use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_hal::{gpio::AnyOutputPin, prelude::Peripherals};
use std_display::serial::Serial;

fn main() -> Result<()> {
    let peripheral = Peripherals::take().expect("never fails");

    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let hertz = 115200;

    let mut serial = Serial::new(uart, tx, rx, enable, hertz);

    loop {
        match serial.send(b"hello!!!") {
            Ok(_) => {
                println!("Sent: hello!!!");
                println!("{:?}", b"hello!!!");
            }
            Err(e) => {
                println!("SendError: {:?}", e);
            }
        }
        let data = match serial.receive() {
            Ok(data) => data,
            Err(e) => {
                println!("ReceiveError: {:?}", e);
                None
            }
        };
        match data {
            Some(t) => {
                println!("Received: {:?}", t);
                println!("Received: {:?}", String::from_utf8(t.to_vec()));
            }
            None => {
                println!("No data received");
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
