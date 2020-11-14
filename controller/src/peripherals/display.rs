use crate::State;
use core::fmt::Write;
use display_interface_spi::SPIInterface;
use embedded_graphics::fonts::{Font6x12, Font8x16};
use embedded_graphics::prelude::*;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::Rgb565,
    prelude::*,
    style::{TextStyle, TextStyleBuilder},
};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v1_compat::OldOutputPin;
use heapless::consts::*;
use heapless::String;
use ili9341::Ili9341;
use ili9341::Orientation;
use micromath::F32Ext;
use nrf52840_hal::gpio::{Output, Pin, PushPull};
use nrf52840_hal::pac::SPIM0;
use nrf52840_hal::spim::{Frequency, Pins, MODE_0};
use nrf52840_hal::Spim;
use ssd1351::builder::Builder;
use ssd1351::interface::SpiInterface;
use ssd1351::mode::GraphicsMode;

pub struct Display {
    display: Ili9341<
        SPIInterface<Spim<SPIM0>, Pin<Output<PushPull>>, Pin<Output<PushPull>>>,
        Pin<Output<PushPull>>,
    >,
}

impl Display {
    pub fn new<D: DelayMs<u16>>(
        spim: SPIM0,
        rst: Pin<Output<PushPull>>,
        dc: Pin<Output<PushPull>>,
        cs: Pin<Output<PushPull>>,
        sck: Pin<Output<PushPull>>,
        mosi: Pin<Output<PushPull>>,
        timer: &mut D,
    ) -> Self {
        let spi_pins = Pins {
            sck,
            mosi: Some(mosi),
            miso: None,
        };
        let spi = Spim::new(spim, spi_pins, Frequency::M16, MODE_0, 0);

        let wrapped_spi = SPIInterface::new(spi, dc, cs);
        let mut display = Ili9341::new(wrapped_spi, rst, timer).expect("display init failed");

        display.set_orientation(Orientation::LandscapeFlipped);

        Self { display }
    }

    pub fn init<D: DelayMs<u8>>(&mut self, timer: &mut D) {}

    pub fn draw_screen(&mut self, state: &State) {
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(Rgb565::YELLOW)
            .background_color(Rgb565::BLUE)
            .build();

        // Create a text at position (20, 30) and draw it using the previously defined style
        match Text::new("Hello Rust!", Point::new(20, 30))
            .into_styled(style)
            .draw(&mut self.display)
        {
            Ok(_) => defmt::info!("Drawed"),
            Err(_) => defmt::info!("Failed!"),
        }

        /*let t = Text::new("Hello Rust!", Point::new(20, 16))
            .into_styled(TextStyle::new(Font8x16, Rgb565::GREEN));
        t.draw(&mut self.display).expect("draw failed");*/

        /*let mut curr_data = String::<U32>::from("Current: ");
        let _ = write!(curr_data, "{}°C", state.current_boiler_temp().round());

        let i: u16 = 0xFFFF;
        self.display.draw(
            Font8x16::render_str(curr_data.as_str())
                .with_stroke(Some(i.into()))
                .into_iter(),
        );

        let mut target_data = String::<U32>::from("Target:  ");
        let _ = write!(target_data, "{}°C", state.target_boiler_temp().round());

        let i: u16 = 0xFFFF;
        self.display.draw(
            Font8x16::render_str(target_data.as_str())
                .with_stroke(Some(i.into()))
                .translate(Coord::new(0, 18))
                .into_iter(),
        );

        let heater_msg = if state.heater_on() {
            "Heater:  On"
        } else {
            "Heater:  Off"
        };

        let i: u16 = 0xFFFF;
        self.display.draw(
            Font8x16::render_str(heater_msg)
                .with_stroke(Some(i.into()))
                .translate(Coord::new(0, 36))
                .into_iter(),
        );

        let mut pid_data = String::<U32>::from("");
        let _ = write!(
            pid_data,
            "P: {} I: {} D: {}",
            state.kp(),
            state.ki(),
            state.kd()
        );
        let i: u16 = 0xFFFF;
        self.display.draw(
            Font6x12::render_str(pid_data.as_str())
                .with_stroke(Some(i.into()))
                .translate(Coord::new(0, 110))
                .into_iter(),
        );*/
    }
}
