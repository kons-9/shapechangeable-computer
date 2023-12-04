// todo: no_std with alloc
// use esp_alloc::EspHeap;
// #[global_allocator]
// static GLOBAL: EspHeap = EspHeap::empty();
// use esp_println::println;
//

use anyhow::Result;
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_hal::prelude::*;

use log::info;

use global_network::DefaultProtocol;

use network_node::header::Header;
use network_node::packet::Packet;
use network_node::utils::util::{self, get_first_messages};
use network_node::NetworkNode;

use ota::ota::Ota;

use std_display::display::Display;
use std_display::efuse::Efuse;
use std_display::serial;

use core::fmt::Write;

use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

#[derive(Debug)]
#[toml_cfg::toml_config]
struct Config {
    #[default = ""]
    url: &'static str,
    #[default = ""]
    filename: &'static str,
    #[default = ""]
    firmware_filename: &'static str,
    #[default = ""]
    wifi_ssid: &'static str,
    #[default = ""]
    wifi_password: &'static str,
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let (efuse, mut display, serial, wifi) = std_display::init().expect("failed to initialize");
    let config = CONFIG;

    macro_rules! display_print {
        ($fmt:expr) => {
            write!(display, $fmt).unwrap();
        };
        ($fmt:expr, $($arg:tt)*) => {
            // display.print(&format_args!($fmt, $($arg)*), false);
            write!(display, $fmt, $($arg)*).unwrap();
        };
    }
    macro_rules! display_println {
        ($fmt:expr) => {
            display_print!($fmt);
            display_print!("\n");
        };
        ($fmt:expr, $($arg:tt)*) => {
            display_print!($fmt, $($arg)*);
            display_print!("\n");
        };
    }

    let ota = Ota::new();
    ota.connect_to_wifi(&mut wifi, &config.wifi_ssid, &config.wifi_password)?;
    if ota.check_firmware_is_latest(&config.url, &config.filename)? {
        info!("Firmware is latest!, no need to update!");
    } else {
        info!("Firmware is not latest!, need to update!");
        ota.download_firmware(&config.url, &config.filename)?;
    }

    let value = efuse.get_efuse_value();
    let first_messages = get_first_messages(value);

    for message in first_messages {
        display_println!("{}", message);
    }

    display_println!("begining network initialization...");

    // network initialization
    let protocol: DefaultProtocol = DefaultProtocol::new();

    let mut network = match NetworkNode::new(serial, protocol, &efuse) {
        Ok(network) => network,
        Err(e) => {
            display_println!("network initialization failed: {:?}", e);
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
    let global_location = network.get_global_location();

    display_println!("network initialized");
    display_println!("(x,y): ({}, {})", coordinate.0, coordinate.1);
    display_println!("gl: {:?}", global_location);
    display_println!("estimation is done.");
    display_println!("waiting for network connection...");

    info!("network initialized");
    info!("coordinate: ({}, {})", coordinate.0, coordinate.1);
    info!("estimation is done");

    let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("../asset/ferris.raw"), 128);
    let image = Image::new(&image_raw, Point::new(0, 0));
    image.draw(&mut display).unwrap();

    info!("complete initialization");

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
