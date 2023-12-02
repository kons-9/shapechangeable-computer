// use std::thread;
use std::thread::sleep;
// import esp_idf's thread

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
use std_display::display2::Display2;
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
    // let hertz = 30_000_000;
    // let hertz = 30_000;
    let hertz = 30.MHz();

    let mut display = Display2::new(spi, sclk, sdo, dc, rst, hertz.into());

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
    let hertz = 115_200;
    let serial = serial::Serial::new(uart, tx, rx, enable, hertz);

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
    let global_location = network.get_global_location();
    let global_location_str = format!("gl: {:?}", global_location);

    display.print("network initialized", true);
    display.print(&coordinate_str, true);
    display.print(&global_location_str, true);
    display.print("estimation is done.", true);
    display.print("waiting for network connection...", true);

    info!("network initialized");
    info!("coordinate: ({}, {})", coordinate.0, coordinate.1);
    info!("estimation is done");

    let coordinate = network.get_coordinate();

    esp_idf_hal::task::thread::ThreadSpawnConfiguration {
        stack_size: 4096,
        priority: 1,
        ..Default::default()
    }
    .set()
    .unwrap();

    let th0 = std::thread::spawn(move || {
        seq!( XCORD in 0..4 {
            seq!( YCORD in 0..4 {
                match coordinate {
                    #(#( (XCORD, YCORD) => {
                        let image_raw: ImageRawLE<Rgb565> =
                            ImageRaw::new(include_bytes!(concat!(
                                // "../../../../python/raw_translater/out/test_",
                                "../asset/test_",
                                XCORD,
                                "_",
                                YCORD,
                                ".raw"
                            )), 128);
                        let image = Image::new(&image_raw, Point::new(0, 0));
                        image.draw(&mut display).unwrap();
                    }, )*)*

                    _ => {
                        display.print("Error: invalid coordinate", true);
                        display.print(&coordinate_str, true);
                        info!("Error: invalid coordinate");
                    }
                }
            });
        });
        loop {}
    });

    info!("waiting for network connection...");

    while let Err(e) = network.flush_all() {
        info!("network connection failed");
        info!("error: {:?}", e);
        sleep(core::time::Duration::from_secs(1));
    }
    // after network connected
    let mut flag = true;
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
