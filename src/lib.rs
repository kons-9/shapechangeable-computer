mod network;
mod sd;
mod display;
mod localnet;
mod efuse;
mod serial;
mod packet;

use anyhow::Result;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::{UartConfig, UartDriver};
use esp_idf_hal::gpio;

use std::{time::Duration, thread::sleep};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
                      
use network::Network;

pub fn run() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Peripherals
    let periperal = Peripherals::take().expect("never fails");

    let uart = periperal.uart1;
    let tx = periperal.pins.gpio21;
    let rx = periperal.pins.gpio20;
    let serial = serial::Serial::new(uart, tx, rx, 115200);

    // setup
    // this include localnet setup
    let mut network = Network::new(serial);

    if network.is_confirmed() {
        root_setup(&mut network)?;
    } else {
        slave_setup(&mut network)?;
    }

    // after network connected
    loop {
        // display process
        unimplemented!();
    }
}
fn root_setup(_network: &mut Network) -> Result<()> {
    Ok(())
}
fn slave_setup(network: &mut Network) -> Result<()> {
    while !network.is_connected() {
        sleep(Duration::from_secs(1));
    }

    network.try_connect()?;
    while !network.can_calculate_coordinate() {
        sleep(Duration::from_secs(1));
    }
    let coordinate = network.calcuate_coordinate()?;
    network.set_coordinate(coordinate);
    
    // then we can connect with root
    network.get_id_from_root()?;
    Ok(())
}

