use anyhow::Result;
use esp_idf_hal::prelude::*;
use network::NetworkNode;
use std::{thread::sleep, time::Duration};
use std_display::display::Display;
use std_display::network::protocol::{DefaultProtocol, Protocol};
use std_display::network::NetworkNode; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use std_display::serial;

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // todo : use interrupt

    // Peripherals
    let periperal = Peripherals::take().expect("never fails");

    let uart = periperal.uart1;
    let tx = periperal.pins.gpio21;
    let rx = periperal.pins.gpio20;
    let serial = serial::Serial::new(uart, tx, rx, 115200);

    let mut protocol: DefaultProtocol = DefaultProtocol::new();
    let mut network = NetworkNode::new(serial);

    if network.is_root() {
        root_setup(&mut network)?;
    } else {
        slave_setup(&mut network)?;
    }

    // set reciever interrupt
    let mut display = Display::new(network.get_coordinate(), network.get_neighbor_coordinate());

    // after network connected
    loop {
        // if there is no packet,

        let packet = {
            let option_packet = network.get_packet(&protocol)?;
            if let None = option_packet {
                sleep(Duration::from_millis(100));
                continue;
            }
            option_packet.unwrap()
        };

        display.draw_image(packet.get_image());
    }
}

fn root_setup(_network: &mut NetworkNode) -> Result<()> {
    Ok(())
}

fn slave_setup(network: &mut NetworkNode) -> Result<()> {
    Ok(())
}
