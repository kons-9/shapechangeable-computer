use anyhow::Result;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use embedded_graphics::prelude::*;
use embedded_graphics::text::{Text, TextStyle};

use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_hal::prelude::*;

use std::{thread::sleep, time::Duration};

use std_display::display::Display;
use std_display::network::protocol::DefaultProtocol;
use std_display::network::NetworkNode;
use std_display::{application, serial};

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Peripherals
    let peripheral = Peripherals::take().expect("never fails");

    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable = peripheral.pins.gpio5;
    let enable: AnyOutputPin = enable.into();
    let serial = serial::Serial::new(uart, tx, rx, enable, 115200);

    let protocol: DefaultProtocol = DefaultProtocol::new();
    let network = NetworkNode::new(serial, protocol)?;
    network.print_coordinate();

    // set reciever interrupt
    let spi = peripheral.spi2;
    let sclk = peripheral.pins.gpio8;
    let dc = PinDriver::output(peripheral.pins.gpio4)?;
    let sdo = peripheral.pins.gpio10;
    let rst = PinDriver::output(peripheral.pins.gpio3)?;
    let hertz = 30.MHz().into();

    let mut display = Display::new(
        spi,
        sclk,
        sdo,
        dc,
        rst,
        hertz,
        network.get_local_location(),
        network.get_global_location(),
        network.get_coordinate(),
    );
    let coordinate = network.get_coordinate();
    let coordinate_str = format!("{} {}", coordinate.0, coordinate.1);

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let text = Text::new(&coordinate_str, Point::new(0, 0), text_style);

    text.draw(&mut display)?;

    println!("estimation is done");

    // after network connected
    loop {
        esp_idf_hal::delay::Delay::delay_ms(1000);
    }
}
