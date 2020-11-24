//! Contains the PID-controlled heater

use crate::pid::{Direction, Pid, Proportional};
use nrf52840_hal::gpio::{Output, Pin, PushPull};
use nrf52840_hal::prelude::*;

pub struct Heater {
    pin: Pin<Output<PushPull>>,
    pid: Pid,
    window_size: u32,
    isr_counter: u32,
    last_output: f32,
}

impl Heater {
    pub fn new(pin: Pin<Output<PushPull>>, config: HeaterConfig) -> Self {
        let window_size = config.window_size;

        let mut pid = Pid::new(
            config.setpoint,
            config.kp,
            config.ki,
            config.kd,
            Proportional::OnMeasurement,
            Direction::Direct,
        );
        pid.set_mode(crate::pid::Mode::Automatic);
        pid.set_sample_time(window_size);
        pid.set_output_limits(0.0, window_size as f32);
        Self {
            pin,
            pid,
            window_size,
            isr_counter: 0,
            last_output: 0.0,
        }
    }

    pub fn control(&mut self, current_temperature: f32) -> Result<bool, HeaterError> {
        if self.last_output <= self.isr_counter as f32 {
            self.turn_heater_off()?;
        } else {
            self.turn_heater_on()?;
        }

        self.isr_counter += 20;
        if self.isr_counter > self.window_size {
            self.isr_counter = 0;
            self.last_output = self.pid.compute(current_temperature).unwrap();

            defmt::info!(
                "Next {{ output: {:f32}, p: {:f32}, i: {:f32}, d: {:f32} }}",
                self.last_output,
                self.pid.get_kp(),
                self.pid.get_ki(),
                self.pid.get_kd(),
            );
        }

        Ok(self.is_on()?)
    }

    pub fn update_pid(&mut self, kp: f32, ki: f32, kd: f32, pon: Proportional) {
        self.pid.set_tunings(kp, ki, kd, pon);
    }

    pub fn is_on(&self) -> Result<bool, HeaterError> {
        self.pin.is_set_high().map_err(|_| HeaterError::PinError)
    }

    fn turn_heater_on(&mut self) -> Result<(), HeaterError> {
        if !self.is_on()? {
            self.pin.set_high().map_err(|_| HeaterError::PinError)?;
        }
        Ok(())
    }

    pub fn turn_heater_off(&mut self) -> Result<(), HeaterError> {
        if self.is_on()? {
            self.pin.set_low().map_err(|_| HeaterError::PinError)?;
        }
        Ok(())
    }
}

pub struct HeaterConfig {
    kp: f32,
    ki: f32,
    kd: f32,
    setpoint: f32,
    window_size: u32,
}

impl HeaterConfig {
    pub fn new(setpoint: f32, kp: f32, ki: f32, kd: f32, window_size: u32) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            window_size,
        }
    }
}

pub enum HeaterError {
    /// Could not read or write from the heater GPIO pin.
    PinError,
}
