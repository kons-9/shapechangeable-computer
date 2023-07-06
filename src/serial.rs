use anyhow::Result;
use esp_idf_hal::gpio;
use esp_idf_hal::gpio::{InputPin, OutputPin};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::{Uart, UartConfig, UartDriver};

pub struct Serial<'d> {
    uart_driver: UartDriver<'d>,
}

impl<'d> Serial<'d> {
    pub fn new<UART: Uart>(
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

        let uart_driver = UartDriver::new(
            uart,
            tx,
            rx,
            Option::<gpio::Gpio0>::None,
            Option::<gpio::Gpio1>::None,
            &config,
        )
        .unwrap();
        Serial { uart_driver }
    }
    pub fn send(&self, data: &[u8]) -> Result<usize> {
        Ok(self.uart_driver.write(data)?)
    }
    pub fn receive(&self) -> Result<Option<[u8; 8]>> {
        // todo arduino rx buffer may overflow, so we need to handle it

        // pull u64 from uart_driver
        let mut buffer = [0; 8];
        let byte = self.uart_driver.read(&mut buffer, 0)?;
        if byte != 8 {
            return Ok(None);
        }
        Ok(Some(buffer))
    }
}
