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

use seq_macro::seq;

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
    display.set_offset(0, 0);

    esp_idf_hal::delay::Delay::delay_ms(10);

    loop {
        seq!(x in 0..4 {
            seq!(y in 0..4 {
                let image_raw: ImageRawLE<Rgb565> =
                    ImageRaw::new(include_bytes!(concat!(
                        "../../../../python/raw_translater/out/test_",
                        x,
                        "_",
                        y,
                        ".raw"
                    )), 126);
                let image = Image::new(&image_raw, Point::new(2, 2));
                image.draw(&mut display).unwrap();
                esp_idf_hal::delay::Delay::delay_ms(1000);
            });
        });
    }
}
