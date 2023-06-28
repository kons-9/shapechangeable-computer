use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::{Uart, UartConfig, UartDriver};
use esp_idf_hal::gpio::{OutputPin, InputPin};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::gpio;
use anyhow::Result;

pub struct Serial<'d> {
    uart_driver: UartDriver<'d>,
}

impl<'d> Serial<'d> {

    pub fn new<UART: Uart> (
            uart: impl Peripheral<P = UART> + 'd,
            tx: impl Peripheral<P = impl OutputPin> + 'd,
            rx: impl Peripheral<P = impl InputPin> + 'd,
            // cts: Option<impl Peripheral<P = impl InputPin> + 'd>,
            // rts: Option<impl Peripheral<P = impl OutputPin> + 'd>,
            hertz: u32,
        ) -> Self {

        // Peripherals
        // let periperal = Peripherals::take().expect("never fails");

        // let tx = periperal.pins.gpio21;
        // let rx = periperal.pins.gpio20;
        let config = UartConfig::default().baudrate(Hertz(hertz));

        let uart_driver = UartDriver::new (
            uart,
            tx,
            rx,
            Option::<gpio::Gpio0>::None,
            Option::<gpio::Gpio1>::None,
            &config
        ).unwrap();
        Serial {uart_driver}
    }
    pub fn send(&self, data: &[u8]) -> Result<usize> {
        Ok(self.uart_driver.write(data)?)
    }
    pub fn receive(&self) -> Result<Vec<u8>> {
        let mut buffer = [0u8; 1024];
        let mut vec = Vec::new();
        loop {
            let len = self.uart_driver.read(&mut buffer, 0)?;
            if len > 0 {
                for i in 0..len {
                    vec.push(buffer[i]);
                }
            } else {
                break;
            }
        }
        Ok(vec)
    }
}
