use crate::State;
use core::fmt::Write;
use display_interface_spi::SPIInterface;
use embedded_graphics::{
    fonts::{Font6x8, Font8x16, Text},
    pixelcolor::Gray4,
    style::TextStyleBuilder,
};
use embedded_graphics::{prelude::*, primitives::Rectangle, style::PrimitiveStyleBuilder};
use embedded_hal::blocking::delay::DelayMs;
use heapless::consts::*;
use heapless::String;
use micromath::F32Ext;
use nrf52840_hal::gpio::{Output, Pin, PushPull};
use nrf52840_hal::pac::SPIM0;
use nrf52840_hal::spim::{Frequency, Pins, MODE_0};
use nrf52840_hal::Spim;
use ssd1327::display::Ssd1327;

type InnerDisplay =
    Ssd1327<SPIInterface<Spim<SPIM0>, Pin<Output<PushPull>>, Pin<Output<PushPull>>>>;

pub struct Display {
    display: InnerDisplay,
    rst: Pin<Output<PushPull>>,
    alive_pixel: bool,
}

impl Display {
    pub fn new(
        spim: SPIM0,
        rst: Pin<Output<PushPull>>,
        dc: Pin<Output<PushPull>>,
        cs: Pin<Output<PushPull>>,
        sck: Pin<Output<PushPull>>,
        mosi: Pin<Output<PushPull>>,
    ) -> Self {
        let spi_pins = Pins {
            sck,
            mosi: Some(mosi),
            miso: None,
        };
        let spi = Spim::new(spim, spi_pins, Frequency::M2, MODE_0, 0);
        let spii = SPIInterface::new(spi, dc, cs);
        let display = Ssd1327::new(spii);

        Self {
            display,
            rst,
            alive_pixel: false,
        }
    }

    pub fn init<D: DelayMs<u8>>(&mut self, timer: &mut D) {
        self.display.reset(&mut self.rst, timer).ok();
        self.display.init().ok();
    }

    pub fn draw_screen(&mut self, state: &State) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Gray4::BLACK)
            .stroke_width(1)
            .fill_color(Gray4::BLACK)
            .build();

        Rectangle::new(Point::zero(), Point::new(127, 127))
            .into_styled(style)
            .draw(&mut self.display)
            .ok();

        let mut curr_data = String::<U32>::from("Current: ");
        let _ = write!(curr_data, "{}°C", state.current_boiler_temp().round());

        let style = TextStyleBuilder::new(Font8x16)
            .text_color(Gray4::WHITE)
            .background_color(Gray4::BLACK)
            .build();

        Text::new(curr_data.as_str(), Point::new(0, 0))
            .into_styled(style)
            .draw(&mut self.display)
            .ok();

        let mut target_data = String::<U32>::from("Target:  ");
        let _ = write!(target_data, "{}°C", state.target_boiler_temp().round());

        let style = TextStyleBuilder::new(Font6x8)
            .text_color(Gray4::WHITE)
            .background_color(Gray4::BLACK)
            .build();

        Text::new(target_data.as_str(), Point::new(0, 30))
            .into_styled(style)
            .draw(&mut self.display)
            .ok();

        let heater_msg = if state.heater_on() {
            "Heater:  On"
        } else {
            "Heater:  Off"
        };

        let style = TextStyleBuilder::new(Font6x8)
            .text_color(Gray4::WHITE)
            .background_color(Gray4::BLACK)
            .build();

        Text::new(heater_msg, Point::new(0, 40))
            .into_styled(style)
            .draw(&mut self.display)
            .ok();

        let mut out_data = String::<U32>::from("PID Output: ");
        let _ = write!(out_data, "{}", state.last_pid_out().round());

        let style = TextStyleBuilder::new(Font6x8)
            .text_color(Gray4::WHITE)
            .background_color(Gray4::BLACK)
            .build();

        Text::new(out_data.as_str(), Point::new(0, 50))
            .into_styled(style)
            .draw(&mut self.display)
            .ok();

        let mut pid_data = String::<U32>::from("");
        let _ = write!(
            pid_data,
            "P: {} I: {} D: {}",
            state.kp(),
            state.ki(),
            state.kd()
        );

        let style = TextStyleBuilder::new(Font6x8)
            .text_color(Gray4::WHITE)
            .background_color(Gray4::BLACK)
            .build();

        Text::new(pid_data.as_str(), Point::new(0, 60))
            .into_styled(style)
            .draw(&mut self.display)
            .ok();

        if self.alive_pixel {
            Text::new("<>", Point::new(0, 100))
                .into_styled(style)
                .draw(&mut self.display)
                .ok();
            self.alive_pixel = false;
        } else {
            Text::new("><", Point::new(0, 100))
                .into_styled(style)
                .draw(&mut self.display)
                .ok();
            self.alive_pixel = true;
        }

        self.display.flush().ok();
    }
}
