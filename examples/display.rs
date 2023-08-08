use anyhow::Result;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::Peripherals;

use std_display::display::Display;
use std_display::font::Colors;
use std_display::network::localnet::LocalNetworkLocation;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio8;
    let sdo = peripherals.pins.gpio10;
    let rst = PinDriver::output(peripherals.pins.gpio3)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;

    let mut display = Display::new(
        spi,
        sclk,
        sdo,
        dc,
        rst,
        115200,
        // for rotation
        LocalNetworkLocation::UpLeft,
        LocalNetworkLocation::UpLeft,
        (0, 0),
    );

    display.set_offset(0, 25);

    esp_idf_hal::delay::Delay::delay_ms(1000);
    let width = 86;
    let data = include_bytes!("../asset/ferris.raw");
    display.draw_image(data, width, Some(Point::new(26, 8)));

    println!("lcd test have done.");
    let mut cnt: u8 = 0;
    loop {
        esp_idf_hal::delay::Delay::delay_ms(1000);
        display.clear(Rgb565::BLACK);
        esp_idf_hal::delay::Delay::delay_ms(1000);
        display.draw_image(data, width, Some(Point::new(26, 8)));
        display.draw_char((cnt % 10) as char, 0, 0, Colors::Red as u16);

        cnt += 1;
    }
}
