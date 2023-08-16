use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use anyhow::Result;
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_hal::prelude::*;
use std_display::efuse::Efuse;

use std::{thread::sleep, time::Duration};

use applications::image::GetImageApp;
use applications::App;
use global_network::DefaultProtocol;
use network_node::NetworkNode;

use std_display::display::Display;
use std_display::serial;

// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // todo : use interrupt

    // Peripherals
    let peripheral = Peripherals::take().expect("never fails");

    // set reciever interrupt
    let spi = peripheral.spi2;
    let sclk = peripheral.pins.gpio8;
    let dc = PinDriver::output(peripheral.pins.gpio4)?;
    let sdo = peripheral.pins.gpio10;
    let rst = PinDriver::output(peripheral.pins.gpio3)?;
    let hertz = 30.MHz().into();

    let mut display = Display::new(spi, sclk, sdo, dc, rst, hertz);
    display.print("display initialized", true);

    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable = peripheral.pins.gpio5;
    let enable: AnyOutputPin = enable.into();
    let serial = serial::Serial::new(uart, tx, rx, enable, 115200);

    let protocol: DefaultProtocol = DefaultProtocol::new();
    let efuse = Efuse::new();
    let mut network = NetworkNode::new(serial, protocol, &efuse)?;
    network.print_coordinate();
    network.join_global_network();

    display.print("network initialized", true);
    display.set_rotation_by_coordinate(
        network.get_local_location(),
        network.get_global_location(),
        network.get_coordinate(),
    );
    display.clear(Rgb565::BLACK)?;

    let mut image_app = GetImageApp::new();
    // after network connected
    loop {
        let messages = {
            let option_messages: Option<Vec<u8>> = network.get_messages()?;
            if let None = option_messages {
                sleep(Duration::from_millis(100));
                continue;
            }
            option_messages.unwrap()
        };
        let (image, width, point) = image_app.process_messages(messages)?;

        let image: ImageRawLE<Rgb565> = ImageRaw::new(&image, width);
        let image = if let Some(point) = point {
            Image::new(&image, point)
        } else {
            Image::new(&image, Point::new(0, 0))
        };
        image.draw(&mut display).unwrap();
    }
}
