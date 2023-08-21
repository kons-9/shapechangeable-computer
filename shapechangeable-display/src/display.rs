use anyhow::Result;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Text;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_hal::digital::v2::OutputPin;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::Gpio0;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi::SpiAnyPins;
use esp_idf_hal::spi::SpiDeviceDriver;
use esp_idf_hal::spi::SpiDriver;
use esp_idf_hal::units::Hertz;

use st7735_lcd;
use st7735_lcd::Orientation;
use st7735_lcd::ST7735;

use network_node::localnet::LocalNetworkLocation;
use network_node::utils::type_alias::Coordinate;

pub struct Display<'d, DC, RST>
where
    // SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    direction: Rotation,
    display: ST7735<SpiDeviceDriver<'d, SpiDriver<'d>>, DC, RST>,
    style: MonoTextStyle<'d, Rgb565>,
    x: i32,
    y: i32,
}

enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    pub fn rotation_to_orientation(&self) -> Orientation {
        // todo: check whether this is correct
        match self {
            Rotation::Zero => Orientation::Portrait,
            Rotation::Ninety => Orientation::Landscape,
            Rotation::OneEighty => Orientation::PortraitSwapped,
            Rotation::TwoSeventy => Orientation::LandscapeSwapped,
        }
    }
}

impl<'d, DC, RST> Display<'d, DC, RST>
where
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new<SPI>(
        spi: impl Peripheral<P = SPI> + 'd,
        sclk: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'd,
        sdo: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'd,
        dc: DC,
        rst: RST,
        baudrate: u32,
    ) -> Display<'d, DC, RST>
    where
        SPI: SpiAnyPins,
    {
        // this project use st7735r
        let (rgb, inverted, width, height) = Self::st7735r_setting();
        let driver_config = Default::default();
        let spi_config = esp_idf_hal::spi::SpiConfig::new().baudrate(Hertz(baudrate));
        let spidriver = esp_idf_hal::spi::SpiDeviceDriver::new_single(
            spi,
            sclk,
            sdo,
            Option::<Gpio0>::None,
            Option::<Gpio0>::None,
            &driver_config,
            &spi_config,
        )
        .unwrap();
        let mut display = ST7735::new(spidriver, dc, rst, rgb, inverted, width, height);

        display.init(&mut FreeRtos).unwrap();
        display.clear(Rgb565::BLACK).unwrap();
        display.set_offset(6, 0);
        Display {
            direction: Rotation::Zero,
            display,
            style: MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
            x: 0,
            y: 10,
        }
    }

    /// return: (rgb, inverted, width, height)
    fn st7735r_setting() -> (bool, bool, u32, u32) {
        (false, false, 128, 128)
    }

    pub fn set_rotation_by_coordinate(
        &mut self,
        local_location: LocalNetworkLocation,
        global_location: LocalNetworkLocation,
        coordinate: Coordinate,
    ) {
        let rotation = Self::calucurate_rotation(local_location, global_location, coordinate);
        self.direction = rotation;
        self.display
            .set_orientation(&self.direction.rotation_to_orientation())
            .unwrap();
    }

    fn calucurate_rotation(
        local_location: LocalNetworkLocation,
        global_location: LocalNetworkLocation,
        coordinate: Coordinate,
    ) -> Rotation {
        // calculate global location

        // todo:
        Rotation::Zero
    }
    pub fn print(&mut self, text: &str, new_line: bool) {
        let x_size = self.style.font.character_size.width as i32;
        let y_size = self.style.font.character_size.height as i32;

        let text_width = text.len() as i32 * x_size;
        // clear the line
        Rectangle::new(
            Point::new(self.x, self.y),
            Size::new(text_width as u32, y_size as u32),
        )
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_fill(
            Rgb565::BLACK,
        ))
        .draw(&mut self.display)
        .unwrap();

        Text::new(text, Point::new(self.x, self.y), self.style)
            .draw(&mut self.display)
            .unwrap();
        if new_line {
            self.x = 0;
            self.y += y_size;
        } else {
            self.x += text_width;
        }
        if self.x > self.display.size().width as i32 {
            self.x = 0;
            self.y += y_size;
        }
        if self.y > self.display.size().height as i32 {
            self.y = 0;
        }
    }

    pub fn set_offset(&mut self, x: u16, y: u16) {
        self.display.set_offset(x, y);
    }
    pub fn clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap()
    }
}

impl<'d, DC, RST> DrawTarget for Display<'d, DC, RST>
where
    DC: OutputPin,
    RST: OutputPin,
{
    type Color = Rgb565;
    type Error = anyhow::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        if let Err(x) = self.display.draw_iter(pixels) {
            println!("draw_iter error: {:?}", x);
            return Err(anyhow::Error::msg("draw_iter error"));
        }
        Ok(())
    }
    fn clear(&mut self, color: Self::Color) -> std::result::Result<(), Self::Error> {
        if let Err(x) = self.display.clear(color) {
            println!("clear error: {:?}", x);
            return Err(anyhow::Error::msg("clear error"));
        }
        Ok(())
    }
}

impl<'d, DC, RST> OriginDimensions for Display<'d, DC, RST>
where
    DC: OutputPin,
    RST: OutputPin,
{
    fn size(&self) -> Size {
        self.display.size()
    }
}
