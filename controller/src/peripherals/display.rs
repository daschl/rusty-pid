use crate::State;
use core::fmt::Write;
use embedded_graphics::fonts::{Font6x12, Font8x16};
use embedded_graphics::prelude::*;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v1_compat::OldOutputPin;
use heapless::consts::*;
use heapless::String;
use micromath::F32Ext;
use nrf52840_hal::gpio::{Output, Pin, PushPull};
use nrf52840_hal::pac::SPIM0;
use nrf52840_hal::spim::{Frequency, Pins, MODE_0};
use nrf52840_hal::Spim;
use ssd1351::builder::Builder;
use ssd1351::interface::SpiInterface;
use ssd1351::mode::GraphicsMode;

pub struct Display {
    display: GraphicsMode<SpiInterface<Spim<SPIM0>, OldOutputPin<Pin<Output<PushPull>>>>>,
    rst: OldOutputPin<Pin<Output<PushPull>>>,
}

impl Display {
    pub fn new(
        spim: SPIM0,
        rst: Pin<Output<PushPull>>,
        dc: Pin<Output<PushPull>>,
        _cs: Pin<Output<PushPull>>,
        sck: Pin<Output<PushPull>>,
        mosi: Pin<Output<PushPull>>,
    ) -> Self {
        let spi_pins = Pins {
            sck,
            mosi: Some(mosi),
            miso: None,
        };
        let spi = Spim::new(spim, spi_pins, Frequency::M8, MODE_0, 0);

        let dc = OldOutputPin::new(dc);
        let display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();

        Self {
            display,
            rst: OldOutputPin::new(rst),
        }
    }

    pub fn init<D: DelayMs<u8>>(&mut self, timer: &mut D) {
        self.display.reset(&mut self.rst, timer);
        self.display.init().unwrap();
    }

    pub fn draw_screen(&mut self, state: &State) {
        let mut curr_data = String::<U32>::from("Current: ");
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
        );
    }
}
