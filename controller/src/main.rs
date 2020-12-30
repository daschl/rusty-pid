#![no_main]
#![cfg_attr(not(test), no_std)]

mod config;
mod peripherals;
mod pid;
mod state;

use cortex_m::peripheral::SCB;
#[allow(unused_imports)]
use defmt_rtt as _;
#[allow(unused_imports)]
use nrf52840_hal as _MemoryLayout;
use pid::Proportional;

use core::sync::atomic::{AtomicUsize, Ordering};
use groundhog_nrf52::GlobalRollingTimer;
use nrf52840_hal::gpio::{p0, p1};
use nrf52840_hal::pac::TIMER1;
use nrf52840_hal::wdt::{count, handles::Hdl0, Parts, Watchdog, WatchdogHandle};
use nrf52840_hal::Timer;
use panic_probe as _;
use peripherals::boiler::Boiler;
use peripherals::display::Display;
use peripherals::heater::{Heater, HeaterConfig};
use state::State;

const ONE_SECOND: i32 = 1_000_000; // schedule is in micros
const HALF_SECOND: i32 = ONE_SECOND / 2;
const TWENTY_MILLIS: i32 = ONE_SECOND / 1000 * 20; // div by 1000 => 1 millis, * 20 => 20 millis

const TARGET_TEMP: f32 = 95.0;

const START_KP: f32 = 250.0;
const START_KI: f32 = 0.03;
const START_KD: f32 = 0.0;

const WARM_KP: f32 = 69.0;
const WARM_KI: f32 = 0.17;
const WARM_KD: f32 = 0.0;

const COLD_ENABLED: bool = true;

#[rtic::app(device = nrf52840_hal::pac, peripherals = true, monotonic = groundhog_nrf52::GlobalRollingTimer)]
const APP: () = {
    struct Resources {
        boiler: Boiler,
        boiler_timer: Timer<TIMER1>,
        heater: Heater,
        display: Display,
        state: State,
        watchdog_handle: WatchdogHandle<Hdl0>,
    }

    #[init(spawn = [boiler_measure_temperature, draw_display, heater_drive_on_off])]
    fn init(ctx: init::Context) -> init::LateResources {
        GlobalRollingTimer::init(ctx.device.TIMER0);

        let port0 = p0::Parts::new(ctx.device.P0);
        let port1 = p1::Parts::new(ctx.device.P1);
        let mut pin_config = config::PinConfig::new(port0, port1);

        // Boiler Sensor Setup
        let sensor_vdd = pin_config.sensor_vdd.take().unwrap();
        let sensor_signal = pin_config.sensor_signal.take().unwrap();

        // Heater Setup
        let heater_signal = pin_config.heater_signal.take().unwrap();

        // Display Setup
        let display_rst_pin = pin_config.display_rst_pin.take().unwrap();
        let display_dc_pin = pin_config.display_dc_pin.take().unwrap();
        let display_cs_pin = pin_config.display_cs_pin.take().unwrap();
        let display_sck_pin = pin_config.display_sck_pin.take().unwrap();
        let display_mosi_pin = pin_config.display_mosi_pin.take().unwrap();

        let target_temp = TARGET_TEMP;
        let kp = START_KP;
        let ki = START_KI;
        let kd = START_KD;

        let heater_config = HeaterConfig::new(target_temp, kp, ki, kd, 1000);

        let boiler_timer = Timer::new(ctx.device.TIMER1);

        let display = Display::new(
            ctx.device.SPIM0,
            display_rst_pin,
            display_dc_pin,
            display_cs_pin,
            display_sck_pin,
            display_mosi_pin,
        );

        let mut heater = Heater::new(heater_signal, heater_config);
        // Turn the heater off after startup for security reasons
        heater.turn_heater_off().ok();

        let mut state = State::new(
            target_temp,
            heater.is_on().ok().unwrap(),
            kp,
            ki,
            kd,
            true,
            false,
        );

        // Watchdog Setup
        let (watchdog_handle, ..) = match Watchdog::try_new(ctx.device.WDT) {
            Ok(mut watchdog) => {
                // Set the watchdog to timeout after 3 seconds (in 32.768kHz ticks)
                watchdog.set_lfosc_ticks(3 * 32768);

                let Parts {
                    watchdog: _watchdog,
                    handles,
                } = watchdog.activate::<count::One>();

                handles
            }
            Err(wdt) => match Watchdog::try_recover::<count::One>(wdt) {
                Ok(Parts { mut handles, .. }) => {
                    defmt::debug!("Watchdog: already active, but recovering!");
                    // Pet the handles on the recovering watchdog early, since it is
                    // already running and counting.
                    handles.0.pet();
                    handles
                }
                Err(_wdt) => {
                    defmt::debug!("Watchdog: already active, resetting!");
                    loop {
                        continue;
                    }
                }
            },
        };

        if ctx.device.POWER.resetreas.read().dog().is_detected() {
            ctx.device.POWER.resetreas.modify(|_r, w| {
                // Clear the watchdog reset reason bit
                w.dog().set_bit()
            });
            defmt::warn!("Controller got restarted by the watchdog!");
            state.set_watchdog_reset(true);
        }

        ctx.spawn.boiler_measure_temperature().ok();
        ctx.spawn.draw_display(true).ok();
        ctx.spawn.heater_drive_on_off().ok();

        init::LateResources {
            boiler: Boiler::new(sensor_signal, sensor_vdd),
            boiler_timer,
            heater,
            display,
            state,
            watchdog_handle,
        }
    }

    #[task(resources= [display, boiler_timer, state], priority = 2, schedule = [draw_display])]
    fn draw_display(ctx: draw_display::Context, init: bool) {
        if init {
            ctx.resources.display.init(ctx.resources.boiler_timer);
        }

        ctx.resources.display.draw_screen(ctx.resources.state);

        ctx.schedule
            .draw_display(ctx.scheduled + ONE_SECOND, false)
            .unwrap();
    }

    #[task(resources = [boiler, boiler_timer, heater, state, watchdog_handle], priority = 2, schedule = [boiler_measure_temperature])]
    fn boiler_measure_temperature(ctx: boiler_measure_temperature::Context) {
        defmt::debug!("Measuring Temperature");

        if let Ok(t) = ctx
            .resources
            .boiler
            .read_temperature(ctx.resources.boiler_timer)
        {
            ctx.resources.state.set_current_boiler_temp(t);

            if COLD_ENABLED
                && t > ctx.resources.state.target_boiler_temp()
                && ctx.resources.state.in_coldstart()
            {
                ctx.resources.state.disable_coldstart();
                ctx.resources
                    .heater
                    .update_pid(WARM_KP, WARM_KI, WARM_KD, Proportional::OnError);
                ctx.resources.state.set_kp(WARM_KP);
                ctx.resources.state.set_ki(WARM_KI);
                ctx.resources.state.set_kd(WARM_KD);
            }
        } else {
            defmt::warn!("Reading temperature failed!");
            // Turn the heater off until we get a good new reading for safety reasons.
            ctx.resources.heater.turn_heater_off().ok();
            ctx.resources.state.set_heater_on(false);
        }

        // Pet the watchdog so it doesn't cause a reset
        ctx.resources.watchdog_handle.pet();

        defmt::debug!("{:?}", ctx.resources.state);

        ctx.schedule
            .boiler_measure_temperature(ctx.scheduled + HALF_SECOND)
            .unwrap();
    }

    #[task(resources = [heater, state], priority = 2, schedule = [heater_drive_on_off])]
    fn heater_drive_on_off(ctx: heater_drive_on_off::Context) {
        let heater_on = ctx
            .resources
            .heater
            .control(ctx.resources.state.current_boiler_temp())
            .ok()
            .unwrap();
        ctx.resources.state.set_heater_on(heater_on);
        ctx.resources
            .state
            .set_last_pid_out(ctx.resources.heater.last_output());

        ctx.schedule
            .heater_drive_on_off(ctx.scheduled + TWENTY_MILLIS)
            .unwrap();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    extern "C" {
        fn SWI0_EGU0();
        fn SWI1_EGU1();
    }
};

#[defmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}

#[defmt::panic_handler]
fn panic() -> ! {
    if cfg!(debug_assertions) {
        cortex_m::asm::udf()
    } else {
        SCB::sys_reset()
    }
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
