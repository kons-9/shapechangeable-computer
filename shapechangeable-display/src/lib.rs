#![no_std]
pub mod display;
pub mod efuse;
pub mod sd;
pub mod serial;

use anyhow::Result;

use esp_idf_hal::gpio::Output;
use esp_idf_hal::gpio::{AnyOutputPin, OutputPin, PinDriver};
use esp_idf_hal::gpio::{Gpio3, Gpio4};
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::SPI2;

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

use crate::display::Display;
use crate::efuse::Efuse;
use crate::serial::Serial;

pub fn init<'d>() -> Result<(
    Efuse,
    Display<'d, PinDriver<'d, Gpio4, Output>, PinDriver<'d, Gpio3, Output>>,
    Serial<'d>,
    BlockingWifi<EspWifi<'d>>,
)> {
    // Peripherals
    let peripheral = Peripherals::take().expect("never fails");
    // display initialization
    let spi: SPI2 = peripheral.spi2;
    let sclk = peripheral.pins.gpio8;
    let dc = PinDriver::output(peripheral.pins.gpio4).expect("failed to set dc pin");
    let sdo = peripheral.pins.gpio10;
    let rst = PinDriver::output(peripheral.pins.gpio3).expect("failed to set rst pin");
    let hertz = 30.MHz().into();

    let mut display = Display::new(spi, sclk, sdo, dc, rst, hertz);

    let efuse = Efuse::new();

    // serial initialization
    let uart = peripheral.uart1;
    let tx = peripheral.pins.gpio21;
    let rx = peripheral.pins.gpio20;
    let enable: AnyOutputPin = peripheral.pins.gpio5.into();
    let serial = serial::Serial::new(uart, tx, rx, enable, 115200);

    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;
    let sys_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripheral.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    return Ok((efuse, display, serial, wifi));
}
