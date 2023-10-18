use std::thread;

use anyhow::Result;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_hal::prelude::*;

use global_network::DefaultProtocol;
use log::info;
use network_node::header::Header;
use network_node::packet::Packet;
use network_node::utils::util::{self, get_first_messages};
use network_node::NetworkNode;
use std_display::display::{Display, Rotation};
use std_display::efuse::Efuse;
use std_display::serial;

use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use seq_macro::seq;

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

    display.set_rotation(Rotation::OneEighty);
    display.set_cursor(0, 20);

    let mac_address = util::get_mac_address(value);
    display.print_with_style(
        "Mac Address:",
        true,
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
    );
    let mac_address_str = format!(" {} = {:05b}", mac_address, mac_address);
    display.print_with_style(
        &mac_address_str,
        true,
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
    );

    let first_messages = get_first_messages(value);

    // serial initialization
    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let serial = serial::Serial::new(uart, tx, rx, enable, 115200);

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

    let coordinate = network.get_coordinate();
    let coordinate_str = format!("(x,y): ({}, {})", coordinate.0, coordinate.1);
    let global_location = network.get_global_location();
    let global_location_str = format!("gl: {:?}", global_location);

    info!("network initialized");
    info!("coordinate: ({}, {})", coordinate.0, coordinate.1);
    info!("estimation is done");

    let coordinate = network.get_coordinate();

    display.print("", true);
    display.print_with_style(
        "Coordinate:",
        true,
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
    );

    let coordinate_str = format!(" ({}, {})", coordinate.0, coordinate.1);
    display.print_with_style(
        &coordinate_str,
        true,
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
    );

    info!("waiting for network connection...");

    // after network connected
    let mut flag = true;
    network.flush_all()?;
    loop {
        // receive data
        let packet = {
            let messages = network.get_packet();
            if messages.is_err() {
                if let Err(e) = network.flush_all() {
                    panic!("flush_read error: {:?}", e);
                }
                continue;
            }
            let messages = messages.unwrap();
            if messages.is_none() {
                if flag {
                    flag = false;
                }
                esp_idf_hal::delay::Delay::delay_ms(500);
                continue;
            }
            messages.unwrap()
        };

        let from = packet.get_global_from();
        esp_idf_hal::delay::Delay::delay_ms(rand::random::<u32>() % 90 + 10);

        match packet.get_header() {
            Header::HRequestConfirmedCoordinate => {
                let packet = Packet::make_confirm_coordinate_packet_by_confirmed_node(
                    network.get_mac_address(),
                    from,
                    network.get_coordinate(),
                    network.get_global_location(),
                )
                .unwrap();

                println!("send packet: {:?}", packet);
                network.send(packet)?;
            }
            _ => {
                println!("received packet: {:?}", packet);
            }
        }
    }
}
