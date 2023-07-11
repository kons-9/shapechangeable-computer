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

pub struct Display {
    direction: Rotation,
}

enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Display {
    pub fn new(coordinate: (i32, i32), neighbor_coordinate: [(i32, i32); 4]) -> Display {
        Display {
            direction: Self::calucurate_rotation(coordinate, neighbor_coordinate),
        }
    }
    pub fn set_rotation_by_coordinate(
        &mut self,
        coordinate: (i32, i32),
        neighbor_coordinate: [(i32, i32); 4],
    ) {
        let rotation = Self::calucurate_rotation(coordinate, neighbor_coordinate);
        self.change_rotation(rotation);
    }
    pub fn draw_image(&self, image: &[u8]) {
        unimplemented!();
    }
    fn calucurate_rotation(
        coordinate: (i32, i32),
        neighbor_coordinate: [(i32, i32); 4],
    ) -> Rotation {
        unimplemented!();
    }
    fn change_rotation(&mut self, rotation: Rotation) {
        self.direction = rotation;
    }
}
