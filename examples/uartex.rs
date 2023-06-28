use anyhow::Result;
use esp_idf_hal::uart::Uart;
use esp_idf_hal::uart::{UartConfig, UartDriver};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::gpio;
use serial::Serial;

fn main() -> Result<()>{
    let periperal = Peripherals::take().expect("never fails");

    let uart = periperal.uart1;
    let tx = periperal.pins.gpio21;
    let rx = periperal.pins.gpio20;
    let config = UartConfig::default().baudrate(Hertz(hertz));

    let mut serial = Serial::new(uart, tx, rx, None, None, 115200);
    serial.send(b"Hello World!\n");

    Ok(())
}
