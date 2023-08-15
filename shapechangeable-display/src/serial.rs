use anyhow::Result;
use esp_idf_hal::gpio::{self, AnyOutputPin, Output, PinDriver};
use esp_idf_hal::gpio::{InputPin, OutputPin};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::{Uart, UartConfig, UartDriver};
use esp_idf_sys::{uart_wait_tx_done, ESP_OK};

use network_node::serial::Serial as SerialTrait;

/// rapper of UartDriver
/// we only read and write 8 bytes because flit size is 8 bytes
/// todo: when a signal is received, push it to the original buffer by interrupt
pub struct Serial<'d> {
    uart_driver: UartDriver<'d>,
    uart_port: esp_idf_sys::uart_port_t,
    enable: PinDriver<'d, AnyOutputPin, Output>,
}

impl<'d> Serial<'d> {
    pub fn new<UART: Uart>(
        uart: impl Peripheral<P = UART> + 'd,
        tx: impl Peripheral<P = impl OutputPin> + 'd,
        rx: impl Peripheral<P = impl InputPin> + 'd,
        enable: impl Peripheral<P = AnyOutputPin> + 'd,
        // cts: Option<impl Peripheral<P = impl InputPin> + 'd>,
        // rts: Option<impl Peripheral<P = impl OutputPin> + 'd>,
        hertz: u32,
    ) -> Self {
        let config = UartConfig::default().baudrate(Hertz(hertz));
        // let enable: AnyOutputPin = enable.into();

        let enable = PinDriver::output(enable).unwrap();

        let uart_driver = UartDriver::new(
            uart,
            tx,
            rx,
            Option::<gpio::Gpio0>::None,
            Option::<gpio::Gpio1>::None,
            &config,
        )
        .unwrap();
        Serial {
            uart_driver,
            uart_port: UART::port(),
            enable,
        }
    }
    /// send [u8; 8] to arduino
    pub fn send(&mut self, data: &[u8; 8]) -> Result<()> {
        // self.enable.set_high()?;
        let length = self.uart_driver.write(data)?;
        // Self::wait_tx_done(self)?;
        // self.enable.set_low()?;
        if length != 8 {
            return Err(anyhow::anyhow!("uart write error"));
        }
        Ok(())
    }
    #[inline]
    fn wait_tx_done(&self) -> Result<()> {
        let ret = unsafe { uart_wait_tx_done(self.uart_port, 1000) };
        if ret == ESP_OK {
            Ok(())
        } else {
            Err(anyhow::anyhow!("uart wait tx done error"))
        }
    }
    /// receive [u8; 8] from arduino
    pub fn receive(&self) -> Result<Option<[u8; 8]>> {
        // todo: arduino rx buffer may overflow, so we need to handle it by interrupt

        // pull u64 from uart_driver
        let mut buffer = [0; 8];
        let byte = self.uart_driver.read(&mut buffer, 0)?;
        if byte != 8 {
            self.flush_read()?;
            return Ok(None);
        }
        Ok(Some(buffer))
    }
    /// flush read buffer
    pub fn flush_read(&self) -> Result<()> {
        Ok(self.uart_driver.flush_read()?)
    }

    /// flush write buffer
    pub fn flush_write(&self) -> Result<()> {
        Ok(self.uart_driver.flush_write()?)
    }
    pub fn flush_all(&self) -> Result<()> {
        self.flush_read()?;
        self.flush_write()?;
        Ok(())
    }
}
impl SerialTrait for Serial<'_> {
    fn send(&mut self, data: &[u8; 8]) -> Result<()> {
        self.send(data)
    }
    fn receive(&mut self) -> Result<Option<[u8; 8]>> {
        self.receive()
    }
    fn flush_read(&mut self) -> Result<()> {
        self.flush_read()
    }
    fn flush_write(&mut self) -> Result<()> {
        self.flush_write()
    }
}
