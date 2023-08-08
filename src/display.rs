use anyhow::Result;
use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::image::ImageRawLE;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::spi::SpiDeviceDriver;
use esp_idf_hal::spi::SpiDriver;

use esp_idf_hal::gpio::Gpio0;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi::SpiAnyPins;
use esp_idf_hal::units::Hertz;
use st7735_lcd;
use st7735_lcd::Orientation;
use st7735_lcd::ST7735;

use crate::font::Font;
use crate::font::Font57;
use crate::id_utils::type_alias::Coordinate;
use crate::network::localnet::LocalNetworkLocation;

pub struct Display<'d, DC, RST>
where
    // SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    direction: Rotation,
    display: ST7735<SpiDeviceDriver<'d, SpiDriver<'d>>, DC, RST>,
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
    pub fn new<S>(
        spi: impl Peripheral<P = S> + 'd,
        sclk: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'd,
        sdo: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'd,
        dc: DC,
        rst: RST,
        baudrate: u32,
        local_location: LocalNetworkLocation,
        global_location: LocalNetworkLocation,
        coordinate: Coordinate,
    ) -> Display<'d, DC, RST>
    where
        S: SpiAnyPins,
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
        let mut ret = Display {
            direction: Rotation::Zero,
            display,
        };

        ret.set_rotation_by_coordinate(local_location, global_location, coordinate);
        ret
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

    pub fn draw_image(&mut self, image: &[u8], width: u32, point: Option<Point>) {
        // convert &[u8] to ImageRawLE<Rgb565>
        let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(image, width);

        let image = match point {
            Some(point) => Image::new(&image_raw, point),
            None => Image::new(&image_raw, Point::zero()),
        };
        image.draw(&mut self.display).unwrap();
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

    pub fn draw_char(&mut self, c: char, x: u16, y: u16, color: u16) {
        let character = Font57::get_char(c);
        let mask = 0x01;
        for row in 0..7 {
            for col in 0..5 {
                let bit = character[col] & (mask << row);

                if bit != 0 {
                    self.display
                        .set_pixel(x - (col as u16), y - (row as u16), color)
                        .unwrap();
                }
            }
        }
    }
    pub fn set_offset(&mut self, x: u16, y: u16) {
        self.display.set_offset(x, y);
    }
    pub fn clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap()
    }
}
