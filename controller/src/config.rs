use nrf52840_hal::gpio::p0::Parts as Parts0;
use nrf52840_hal::gpio::p1::Parts as Parts1;
use nrf52840_hal::gpio::{Floating, Input, Level, Output, Pin, PushPull};

pub struct PinConfig {
    pub sensor_vdd: Option<Pin<Output<PushPull>>>,
    pub sensor_signal: Option<Pin<Input<Floating>>>,
    pub heater_signal: Option<Pin<Output<PushPull>>>,
    pub display_rst_pin: Option<Pin<Output<PushPull>>>,
    pub display_dc_pin: Option<Pin<Output<PushPull>>>,
    pub display_cs_pin: Option<Pin<Output<PushPull>>>,
    pub display_sck_pin: Option<Pin<Output<PushPull>>>,
    pub display_mosi_pin: Option<Pin<Output<PushPull>>>,
}

impl PinConfig {
    #[cfg(feature = "board-dk")]
    pub fn new(p0: Parts0, p1: Parts1) -> Self {
        let sensor_vdd = Some(p1.p1_07.into_push_pull_output(Level::Low).degrade());
        let sensor_signal = Some(p1.p1_08.into_floating_input().degrade());
        let heater_signal = Some(p0.p0_10.into_push_pull_output(Level::Low).degrade());
        let display_rst_pin = Some(p1.p1_14.into_push_pull_output(Level::Low).degrade());
        let display_dc_pin = Some(p1.p1_13.into_push_pull_output(Level::Low).degrade());
        let display_cs_pin = Some(p1.p1_12.into_push_pull_output(Level::Low).degrade());
        let display_sck_pin = Some(p1.p1_11.into_push_pull_output(Level::Low).degrade());
        let display_mosi_pin = Some(p1.p1_10.into_push_pull_output(Level::Low).degrade());

        Self {
            sensor_signal,
            sensor_vdd,
            heater_signal,
            display_rst_pin,
            display_dc_pin,
            display_cs_pin,
            display_sck_pin,
            display_mosi_pin,
        }
    }

    #[cfg(feature = "board-bluefruit")]
    pub fn new(p0: Parts0, p1: Parts1) -> Self {
        let sensor_vdd = Some(p1.p1_09.into_push_pull_output(Level::Low).degrade());
        let sensor_signal = Some(p0.p0_08.into_floating_input().degrade());
        let heater_signal = Some(p0.p0_06.into_push_pull_output(Level::Low).degrade());
        let display_rst_pin = Some(p0.p0_28.into_push_pull_output(Level::Low).degrade());
        let display_dc_pin = Some(p0.p0_02.into_push_pull_output(Level::Low).degrade());
        let display_cs_pin = Some(p0.p0_03.into_push_pull_output(Level::Low).degrade());
        let display_sck_pin = Some(p0.p0_14.into_push_pull_output(Level::Low).degrade());
        let display_mosi_pin = Some(p0.p0_13.into_push_pull_output(Level::Low).degrade());

        Self {
            sensor_signal,
            sensor_vdd,
            heater_signal,
            display_rst_pin,
            display_dc_pin,
            display_cs_pin,
            display_sck_pin,
            display_mosi_pin,
        }
    }
}
