use anyhow::Result;
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_hal::prelude::*;

use global_network::DefaultProtocol;
use log::info;
use network_node::header::Header;
use network_node::packet::Packet;
use network_node::utils::util::{self, get_first_messages};
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

    let efuse = Efuse::new();
    let value = efuse.get_efuse_value();

    let first_messages = get_first_messages(value);
    for message in first_messages {
        display.print(&message, true);
    }

    // serial initialization
    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let serial = serial::Serial::new(uart, tx, rx, enable, 115200);

    display.print("begining network initialization...", true);

    // network initialization
    let protocol: DefaultProtocol = DefaultProtocol::new();

    let mut network = match NetworkNode::new(serial, protocol, &efuse) {
        Ok(network) => network,
        Err(e) => {
            display.print(&format!("failed: {:?}", e), true);
            println!("network initialization failed: {:?}", e);
            loop {}
        }
    };

    network.print_coordinate();
    display.set_rotation_by_coordinate(
        network.get_local_location(),
        network.get_global_location(),
        network.get_coordinate(),
    );

    let coordinate = network.get_coordinate();
    let coordinate_str = format!("(x,y): ({}, {})", coordinate.0, coordinate.1);
    display.print("network initialized", true);
    display.print(&coordinate_str, true);
    display.print("estimation is done.", true);
    display.print("waiting for network connection...", true);

    info!("network initialized");
    info!("coordinate: ({}, {})", coordinate.0, coordinate.1);
    info!("estimation is done");

    // after network connected
    let mut flag = true;
    loop {
        // receive data
        let packet = {
            let messages = network.get_packet();
            if messages.is_err() {
                if let Err(e) = network.flush_all() {
                    display.print(&format!("flush_read error: {:?}", e), true);
                    panic!("flush_read error: {:?}", e);
                }
                display.print("recovering...", true);
                // display.print("flush_read error", true);
                // esp_idf_hal::delay::Delay::delay_ms(10);
                continue;
            }
            let messages = messages.unwrap();
            if messages.is_none() {
                if flag {
                    display.print("no packet", true);
                    flag = false;
                }
                esp_idf_hal::delay::Delay::delay_ms(10);
                continue;
            }
            messages.unwrap()
        };

        let from = packet.get_global_from();
        if util::is_same_localnet(from, network.get_ip_address()) {
            continue;
        }
        flag = true;
        display.print(format!("hdr: {:?}", packet.get_header()).as_str(), true);
        display.print(format!("src: {:?}", from).as_str(), true);

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
