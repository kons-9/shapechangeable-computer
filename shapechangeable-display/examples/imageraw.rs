use anyhow::Result;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::Gpio0;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;

use tinybmp::Bmp;

use embedded_graphics::{
    image::{Image, ImageRaw},
    prelude::*,
};
use st7735_lcd;
use st7735_lcd::Orientation;

#[rustfmt::skip]
const DATA: &[u8] = &[
    0b11101111, 0b0101_0000,
    0b10001000, 0b0101_0000,
    0b11101011, 0b0101_0000,
    0b10001001, 0b0101_0000,
    0b11101111, 0b0101_0000,
];

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio8;
    let sdo = peripherals.pins.gpio10;
    let sdi = Option::<Gpio0>::None;
    let cs = Option::<Gpio0>::None;
    let driver_config = Default::default();
    let hertz = Hertz(30_000_000);
    let spi_config = spi::SpiConfig::new().baudrate(hertz);
    let spi =
        spi::SpiDeviceDriver::new_single(spi, sclk, sdo, sdi, cs, &driver_config, &spi_config)?;

    let rst = PinDriver::output(peripherals.pins.gpio3)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;

    let rgb = false;
    let inverted = false;
    let width = 128;
    let height = 128;

    let mut delay = FreeRtos;

    let mut display = st7735_lcd::ST7735::new(spi, dc, rst, rgb, inverted, width, height);

    display.init(&mut delay).unwrap();
    display.clear(Rgb565::BLACK).unwrap();
    display
        .set_orientation(&Orientation::LandscapeSwapped)
        .unwrap();
    display.set_offset(0, 25);

    esp_idf_hal::delay::Delay::delay_ms(1000);
    // The image dimensions and the format of the stored raw data must be specified
    // when the `new` function is called. The data format can, for example, be specified
    // by using the turbofish syntax. For the image dimensions only the width must be
    // passed to the `new` function. The image height will be calculated based on the
    // length of the image data and the data format.
    let bmp_data = include_bytes!("../asset/hello.bmp");

    // Parse the BMP file.
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();

    let image = Image::new(&bmp, Point::zero());

    image.draw(&mut display);

    println!("Done");
    loop {
        esp_idf_hal::delay::Delay::delay_ms(1000);
    }
}
