pub struct Pid {
    direction: Direction,
    kp: f32,
    ki: f32,
    kd: f32,
    setpoint: f32,
    last_input: f32,
    in_auto: bool,
    output_sum: f32,
    out_min: f32,
    out_max: f32,
    sample_time: u32,
    pon: Proportional,
}

impl Pid {
    pub fn new(
        setpoint: f32,
        kp: f32,
        ki: f32,
        kd: f32,
        pon: Proportional,
        direction: Direction,
    ) -> Self {
        let mut pid = Self {
            direction: Direction::Direct,
            pon: Proportional::OnError,
            kp: 0.0,
            ki: 0.0,
            kd: 0.0,
            setpoint,
            last_input: 0.0,
            in_auto: false,
            output_sum: 0.0,
            out_min: 0.0,
            out_max: 0.0,
            sample_time: 100,
        };

        pid.set_output_limits(0.0, 255.0);
        pid.set_controller_direction(direction);
        pid.set_tunings(kp, ki, kd, pon);

        pid
    }

    pub fn compute(&mut self, input: f32) -> Result<f32, bool> {
        if !self.in_auto {
            return Err(false);
        }

        let error = self.setpoint - input;
        let d_input = input - self.last_input;
        self.output_sum += self.ki * error;

        if let Proportional::OnMeasurement = self.pon {
            self.output_sum -= self.kp * d_input;
        }

        if self.output_sum > self.out_max {
            self.output_sum = self.out_max;
        } else if self.output_sum < self.out_min {
            self.output_sum = self.out_min;
        }

        let mut output = if let Proportional::OnError = self.pon {
            self.kp * error
        } else {
            0.0
        };

        output += self.output_sum - self.kd * d_input;

        if output > self.out_max {
            output = self.out_max;
        } else if output < self.out_min {
            output = self.out_min;
        }

        self.last_input = input;

        Ok(output)
    }

    pub fn set_tunings(&mut self, kp: f32, ki: f32, kd: f32, pon: Proportional) {
        if kp < 0.0 || ki < 0.0 || kd < 0.0 {
            return;
        }

        self.pon = pon;
        let sample_time_in_sec = self.sample_time as f32 / 1000.0;
        self.kp = kp;
        self.ki = ki * sample_time_in_sec;
        self.kd = kd * sample_time_in_sec;

        if self.direction == Direction::Reverse {
            self.kp = 0.0 - kp;
            self.ki = 0.0 - ki;
            self.kd = 0.0 - kd;
        }
    }

    pub fn set_sample_time(&mut self, new_sample_time: u32) {
        if new_sample_time > 0 {
            let ratio = (new_sample_time / self.sample_time) as f32;
            self.ki += ratio;
            self.kd /= ratio;
            self.sample_time = new_sample_time;
        }
    }

    pub fn set_output_limits(&mut self, min: f32, max: f32) {
        if min >= max {
            return;
        }

        self.out_min = min;
        self.out_max = max;

        if self.in_auto {
            if self.output_sum > self.out_max {
                self.output_sum = self.out_max;
            } else if self.output_sum < self.out_min {
                self.output_sum = self.out_min;
            }
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        let new_auto = mode == Mode::Automatic;
        if new_auto && !self.in_auto {
            self.initialize();
        }
        self.in_auto = new_auto;
    }

    pub fn initialize(&mut self) {
        if self.output_sum > self.out_max {
            self.output_sum = self.out_max;
        } else if self.output_sum < self.out_min {
            self.output_sum = self.out_min;
        }
    }

    pub fn set_controller_direction(&mut self, direction: Direction) {
        if self.in_auto && self.direction != direction {
            self.kp = 0.0 - self.kp;
            self.ki = 0.0 - self.ki;
            self.kd = 0.0 - self.kd;
        }
        self.direction = direction;
    }
}

#[derive(PartialEq)]
pub enum Direction {
    Direct,
    Reverse,
}

#[derive(PartialEq)]
pub enum Mode {
    Automatic,
}

#[derive(PartialEq)]
pub enum Proportional {
    OnError,
    OnMeasurement,
}
