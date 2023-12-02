use anyhow::Result;
use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRawLE;
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
use esp_idf_hal::gpio::Gpio10;
use esp_idf_hal::gpio::Gpio3;
use esp_idf_hal::gpio::Gpio4;
use esp_idf_hal::gpio::Gpio8;
use esp_idf_hal::gpio::Output;
use esp_idf_hal::gpio::PinDriver;
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

use core::fmt::Write;

pub struct Display2<'d> {
    direction: Rotation,
    display: ST7735<
        SpiDeviceDriver<'d, SpiDriver<'d>>,
        PinDriver<'d, Gpio4, Output>,
        PinDriver<'d, Gpio3, Output>,
    >,
    style: MonoTextStyle<'d, Rgb565>,
    x: i32,
    y: i32,
}

pub enum Rotation {
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

impl<'d> Display2<'d> {
    pub fn new(
        // spi: impl Peripheral<P = SPI> + 'd,
        spi: esp_idf_hal::spi::SPI2,
        // sclk: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'd,
        sclk: Gpio8,
        // sdo: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'd,
        sdo: Gpio10,
        dc: PinDriver<'d, Gpio4, Output>,
        rst: PinDriver<'d, Gpio3, Output>,
        baudrate: Hertz,
    ) -> Display2<'d> {
        // this project use st7735r
        let (rgb, inverted, width, height) = Self::st7735r_setting();
        let driver_config = Default::default();
        let spi_config = esp_idf_hal::spi::SpiConfig::new().baudrate(baudrate);
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
        Display2 {
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
        self.set_rotation(rotation);
    }

    pub fn set_rotation(&mut self, rotation: Rotation) {
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
        Rotation::OneEighty
    }
    pub fn print_with_style(
        &mut self,
        text: &str,
        new_line: bool,
        style: MonoTextStyle<'d, Rgb565>,
    ) {
        let prev_style = self.style;
        self.style = style;
        self.print(text, new_line);
        self.style = prev_style;
    }

    pub fn print(&mut self, text: &str, new_line: bool) {
        let x_size = self.style.font.character_size.width as i32;
        let y_size = self.style.font.character_size.height as i32;

        let text_width = text.len() as i32 * x_size;
        // todo: new line if self.x + text_width > display.width

        // clear the line
        Rectangle::new(
            Point::new(self.x, self.y - y_size + 1),
            // Size::new((text_width + x_size * 2) as u32, y_size as u32),
            Size::new(self.display.size().width, y_size as u32),
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
            self.y = y_size;
        }
    }
    pub fn depict(&mut self, image: &Image<'_, ImageRawLE<Rgb565>>) {
        image.draw(&mut self.display).expect("failed to draw image");
    }

    pub fn set_offset(&mut self, x: u16, y: u16) {
        self.display.set_offset(x, y);
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap();
        self.set_cursor(0, 10);
    }

    pub fn reset(&mut self) {
        self.clear(Rgb565::BLACK);
    }

    pub fn set_cursor(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

impl<'d> DrawTarget for Display2<'d> {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels)
    }
    fn clear(&mut self, color: Self::Color) -> core::result::Result<(), Self::Error> {
        self.display.clear(color)
    }
    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.display.fill_solid(area, color)
    }
    // fn fill_contiguous<I>(&mut self, area: &Rectangle, color: I) -> Result<(), Self::Error>
    // where
    //     I: IntoIterator<Item = Self::Color>,
    // {
    //     self.display.fill_contiguous(area, color)
    // }
}

impl<'d> core::fmt::Write for Display2<'d> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s, false);
        Ok(())
    }
}

impl<'d> OriginDimensions for Display2<'d> {
    fn size(&self) -> Size {
        self.display.size()
    }
}
