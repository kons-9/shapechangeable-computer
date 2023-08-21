use anyhow::Result;
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_hal::prelude::*;

use global_network::DefaultProtocol;
use log::info;
use network_node::header::Header;
use network_node::packet::Packet;
use network_node::NetworkNode;
use std_display::display::Display;
use std_display::efuse::Efuse;
use std_display::serial;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Peripherals
    let peripheral = Peripherals::take().expect("never fails");
    // display initialization
    let spi = peripheral.spi2;
    let sclk = peripheral.pins.gpio8;
    let dc = PinDriver::output(peripheral.pins.gpio4).expect("failed to set dc pin");
    let sdo = peripheral.pins.gpio10;
    let rst = PinDriver::output(peripheral.pins.gpio3).expect("failed to set rst pin");
    let hertz = 30.MHz().into();

    let mut display = Display::new(spi, sclk, sdo, dc, rst, hertz);

    display.print("display initialized", true);
    display.print("begining serial initialization...", true);

    // serial initialization
    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let serial = serial::Serial::new(uart, tx, rx, enable, 115200);

    display.print("serial initialized", true);
    display.print("begining network initialization...", true);

    // network initialization
    let protocol: DefaultProtocol = DefaultProtocol::new();
    let efuse = Efuse::new();
    let mut network =
        NetworkNode::new(serial, protocol, &efuse).expect("network initialization failed");
    network.print_coordinate();
    display.set_rotation_by_coordinate(
        network.get_local_location(),
        network.get_global_location(),
        network.get_coordinate(),
    );

    info!("network initialized");
    display.print("network initialized", true);

    let coordinate = network.get_coordinate();
    let coordinate_str = format!("coordinate: ({}, {})", coordinate.0, coordinate.1);
    display.print(&coordinate_str, true);
    display.print("estimation is done.", true);
    display.print("waiting for network connection...", true);

    info!("coordinate: ({}, {})", coordinate.0, coordinate.1);
    info!("estimation is done");

    // after network connected
    loop {
        // receive data
        let packet = {
            let messages = network.get_packet();
            if messages.is_err() {
                network.flush_read().expect("hardware error");
                display.print("flush_read error", true);
                esp_idf_hal::delay::Delay::delay_ms(100);
                continue;
            }
            let messages = messages.unwrap();
            if messages.is_none() {
                display.print("no packet", true);
                esp_idf_hal::delay::Delay::delay_ms(100);
                continue;
            }
            messages.unwrap()
        };
        display.print("received packet", true);

        match packet.get_header() {
            Header::HCheckConnection => {
                network.send(Packet::make_check_connection_packet(
                    network.get_ip_address(),
                ))?;
            }
            Header::HRequestConfirmedCoordinate => {
                let id = network.get_ip_address() as u16;
                let x = network.get_coordinate().0 as u16;
                let y = network.get_coordinate().1 as u16;

                let data = [
                    0b11111111 as u8,
                    (id >> 8) as u8,
                    (id & 0xff) as u8,
                    (x >> 8) as u8,
                    (x & 0xff) as u8,
                    (y >> 8) as u8,
                    (y & 0xff) as u8,
                ]
                .to_vec();
                network.send(Packet::new(
                    0,
                    Header::ConfirmCoordinate,
                    network.get_ip_address(),
                    network_node::packet::ToId::Broadcast,
                    network.get_ip_address(),
                    network_node::packet::ToId::Broadcast,
                    data,
                ))?;
            }
            _ => {
                println!("received packet: {:?}", packet);
            }
        }
    }
}
