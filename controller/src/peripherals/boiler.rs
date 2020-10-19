//! Encapsulates the Boiler Peripheral

use embedded_hal::blocking::delay::DelayUs;
use nrf52840_hal::gpio::{Floating, Input, Output, Pin, PushPull};
use tsic::{SensorType, Tsic, TsicError};

pub type BoilerTemperature = f32;

pub struct Boiler {
    temp_sensor: Tsic<Pin<Input<Floating>>, Pin<Output<PushPull>>>,
    last_temp: Option<BoilerTemperature>,
}

impl Boiler {
    pub fn new(signal_pin: Pin<Input<Floating>>, vdd_pin: Pin<Output<PushPull>>) -> Self {
        let temp_sensor = Tsic::with_vdd_control(SensorType::Tsic306, signal_pin, vdd_pin);
        Self {
            temp_sensor,
            last_temp: None,
        }
    }

    pub fn read_temperature<D: DelayUs<u8>>(
        &mut self,
        delay: &mut D,
    ) -> Result<BoilerTemperature, BoilerError> {
        match self.temp_sensor.read(delay) {
            Ok(t) => {
                let rounded = t.as_celsius() as BoilerTemperature;
                self.last_temp = Some(rounded);
                Ok(rounded)
            }
            Err(e) => Err(BoilerError::TempReadFailed { cause: e }),
        }
    }

    pub fn current_temperature(&self) -> Option<BoilerTemperature> {
        self.last_temp
    }
}

pub enum BoilerError {
    TempReadFailed { cause: TsicError },
}
