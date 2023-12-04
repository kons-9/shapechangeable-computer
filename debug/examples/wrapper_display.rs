// use std::thread;
use std::thread::sleep;
use std::time::Duration;
// import esp_idf's thread

use anyhow::Result;
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_hal::prelude::*;

use debug::display::Display;
use debug::display2::Display2;
use debug::efuse::Efuse;
use debug::serial;
use global_network::DefaultProtocol;
use log::info;
use network_node::header::Header;
use network_node::packet::Packet;
use network_node::utils::util::{self, get_first_messages};
use network_node::NetworkNode;

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

    let image = ImageRawLE::new(include_bytes!("../asset/ferris.raw"), 86);
    let image = Image::new(&image, Point::new(0, 0));

    let iamge2 = ImageRawLE::new(include_bytes!("../asset/test_0_1.raw"), 128);
    let iamge2 = Image::new(&iamge2, Point::new(0, 0));

    loop {
        display.depict(&image);
        sleep(Duration::from_millis(1000));
        display.clear(Rgb565::BLACK);
        sleep(Duration::from_millis(1000));

        image.draw(&mut display);
        sleep(Duration::from_millis(1000));
        display.clear(Rgb565::BLACK);
        sleep(Duration::from_millis(1000));

        display.depict(&iamge2);
        sleep(Duration::from_millis(1000));
        display.clear(Rgb565::BLACK);
        sleep(Duration::from_millis(1000));

        iamge2.draw(&mut display);
        sleep(Duration::from_millis(1000));
        display.clear(Rgb565::BLACK);
        sleep(Duration::from_millis(1000));
    }
}
