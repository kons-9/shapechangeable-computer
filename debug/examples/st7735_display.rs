use anyhow::Result;
use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::Gpio0;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use st7735_lcd;
use st7735_lcd::Orientation;

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

    let mut display = st7735_lcd::ST7735::new(spi, dc, rst, rgb, inverted, width, height);

    display.init(&mut FreeRtos).unwrap();
    display.clear(Rgb565::BLACK).unwrap();
    display
        .set_orientation(&Orientation::PortraitSwapped)
        .unwrap();
    display.set_offset(6, 0);
    // display.set_cursor(0, 10);

    esp_idf_hal::delay::Delay::delay_ms(1000);

    const fn bytes_per_row(width: u32, bits_per_pixel: usize) -> usize {
        (width as usize * bits_per_pixel + 7) / 8
    }

    // let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("../asset/ferris.raw"), 86);
    // let image = Image::new(&image_raw, Point::new(26, 8));
    println!(
        "bytes_per_pixel: {}",
        <Rgb565 as PixelColor>::Raw::BITS_PER_PIXEL
    );

    let image = include_bytes!("../asset/test_2_1.raw");
    println!("image len: {}", image.len());
    println!("image width: {}", 128);
    println!(
        "image height: {}",
        image.len() / bytes_per_row(128, <Rgb565 as PixelColor>::Raw::BITS_PER_PIXEL)
    );

    let image = include_bytes!("../asset/ferris.raw");
    println!("image len: {}", image.len());
    println!("image width: {}", 86);
    println!(
        "image height: {}",
        image.len() / bytes_per_row(86, <Rgb565 as PixelColor>::Raw::BITS_PER_PIXEL)
    );

    let image_raw: ImageRawLE<Rgb565> =
        // ImageRaw::new(include_bytes!("../asset/test_2_1.raw"), 128 + 2);
        ImageRaw::new(include_bytes!("../asset/test_2_1.raw"), 128);
    let image = Image::new(&image_raw, Point::new(0, 0));
    image.draw(&mut display).unwrap();

    println!("lcd test have done.");
    loop {
        esp_idf_hal::delay::Delay::delay_ms(1000);
        display.clear(Rgb565::BLACK).unwrap();
        esp_idf_hal::delay::Delay::delay_ms(1000);
        image.draw(&mut display).unwrap();
    }
}
