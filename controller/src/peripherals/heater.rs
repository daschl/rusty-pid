//! Contains the PID-controlled heater

use nrf52840_hal::gpio::{Output, Pin, PushPull};
use nrf52840_hal::prelude::*;
use pid::Pid;

const PID_P_LIMIT: f32 = 1000.0;
const PID_I_LIMIT: f32 = 1000.0;
const PID_D_LIMIT: f32 = 0.0;

pub struct Heater {
    pin: Pin<Output<PushPull>>,
    config: HeaterConfig,
    pid: Pid<f32>,
}

impl Heater {
    pub fn new(pin: Pin<Output<PushPull>>, config: HeaterConfig) -> Self {
        let pid = Pid::new(
            config.kp,
            config.ki,
            config.kd,
            PID_P_LIMIT,
            PID_I_LIMIT,
            PID_D_LIMIT,
            config.setpoint,
        );
        Self { pin, pid, config }
    }

    pub fn control(&mut self, current_temperature: f32) -> Result<bool, HeaterError> {
        let next = self.pid.next_control_output(current_temperature);
        if next.output > self.config.setpoint {
            self.turn_heater_on()?;
        } else {
            self.turn_heater_off()?;
        }
        Ok(self.is_on()?)
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
}

impl HeaterConfig {
    pub fn new(setpoint: f32, kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
        }
    }
}

pub enum HeaterError {
    /// Could not read or write from the heater GPIO pin.
    PinError,
}
