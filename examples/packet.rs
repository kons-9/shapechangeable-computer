use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_hal::prelude::Peripherals;
use std_display::efuse::Efuse;
use std_display::network::packet::{self, Packet, ToId};
use std_display::serial::Serial;

fn main() -> Result<()> {
    let periperal = Peripherals::take().expect("never fails");

    let uart = periperal.uart1;
    let tx = periperal.pins.gpio21;
    let rx = periperal.pins.gpio20;
    let hertz = 115200;

    let serial = Serial::new(uart, tx, rx, hertz);

    let ip_address = Efuse::new().get_efuse_value();
    let packet = packet::Packet::new(
        0,
        std_display::network::header::Header::HCheckConnection,
        ip_address,
        ToId::Broadcast,
        ip_address,
        ToId::Broadcast,
        vec![],
    );
    loop {
        match packet.send(&serial) {
            Ok(_) => {
                println!("check connection");
            }
            Err(e) => {
                println!("SendError: {:?}", e);
            }
        }
        thread::sleep(Duration::from_secs(1));
        let data = match Packet::receive(&serial) {
            Ok(data) => data,
            Err(e) => {
                println!("ReceiveError: {:?}", e);
                None
            }
        };
        match data {
            Some(t) => {
                println!("Received: {:?}", t);
                let header = t.get_header();
                println!("Received: {:?}", header);
            }
            None => {
                println!("No data received");
            }
        }
    }
}
