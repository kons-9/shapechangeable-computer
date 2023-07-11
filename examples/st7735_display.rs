use anyhow::Result;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use esp_idf_hal::delay::Delay;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::Gpio0;
use esp_idf_hal::gpio::Gpio1;
use esp_idf_hal::gpio::{self, PinDriver};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;
use esp_idf_hal::uart::Uart;
use esp_idf_hal::uart::{UartConfig, UartDriver};

use st7735_lcd;
use st7735_lcd::Orientation;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio3;
    let sdo = peripherals.pins.gpio4;
    let sdi = Option::<Gpio0>::None;
    let cs = Option::<Gpio0>::None;
    let driver_config = Default::default();
    let spi_config = spi::SpiConfig::new().baudrate(4.MHz().into());
    let spi =
        spi::SpiDeviceDriver::new_single(spi, sclk, sdo, sdi, cs, &driver_config, &spi_config)?;

    let rst = PinDriver::output(peripherals.pins.gpio7)?;
    let dc = PinDriver::output(peripherals.pins.gpio8)?;

    let rgb = true;
    let inverted = false;
    let width = 128;
    let height = 128;

    let mut delay = FreeRtos;

    let mut display = st7735_lcd::ST7735::new(spi, dc, rst, rgb, inverted, width, height);

    display.init(&mut delay).unwrap();
    display.set_orientation(&Orientation::Landscape).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    esp_idf_hal::delay::Delay::delay_ms(1000);
    display.clear(Rgb565::RED).unwrap();
    display.set_offset(0, 25);

    println!("lcd test have done.");
    loop {
        esp_idf_hal::delay::Delay::delay_ms(1000);
    }
}
