#![no_main]
#![cfg_attr(not(test), no_std)]

mod peripherals;

#[allow(unused_imports)]
use defmt_rtt as _GlobalLogger;
#[allow(unused_imports)]
use nrf52840_hal as _MemoryLayout;

use core::sync::atomic::{AtomicUsize, Ordering};
use nrf52840_hal::clocks::{Clocks, HFCLK_FREQ as CPU_FREQ};
use nrf52840_hal::gpio::{p0, p1, Level};
use nrf52840_hal::pac::TIMER1;
use nrf52840_hal::Timer;
use peripherals::bluetooth::Bluetooth;
use peripherals::boiler::Boiler;
use peripherals::heater::{Heater, HeaterConfig};
use rtic::cyccnt::U32Ext;
use rubble::link::queue::SimpleQueue;
use rubble::link::MIN_PDU_BUF;
use rubble_nrf5x::radio::PacketBuffer;

const ONE_SECOND: u32 = CPU_FREQ; // 1s => CPU runs at 64 mhz

#[rtic::app(device = nrf52840_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        #[init([0; MIN_PDU_BUF])]
        ble_tx_buf: PacketBuffer,
        #[init([0; MIN_PDU_BUF])]
        ble_rx_buf: PacketBuffer,
        #[init(SimpleQueue::new())]
        tx_queue: SimpleQueue,
        #[init(SimpleQueue::new())]
        rx_queue: SimpleQueue,
        bluetooth: Bluetooth,
        boiler: Boiler,
        boiler_timer: Timer<TIMER1>,
        heater: Heater,
    }

    #[init(spawn = [boiler_measure_temperature], resources = [ble_tx_buf, ble_rx_buf, tx_queue, rx_queue])]
    fn init(mut ctx: init::Context) -> init::LateResources {
        // Preparations Needed for Bluetooth
        let _ = Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();

        // Boiler Sensor Setup
        let port1 = p1::Parts::new(ctx.device.P1);
        let sensor_vdd = port1.p1_07.into_push_pull_output(Level::Low).degrade();
        let sensor_signal = port1.p1_08.into_floating_input().degrade();

        // Heater Setup
        let port0 = p0::Parts::new(ctx.device.P0);
        let heater_signal = port0.p0_10.into_push_pull_output(Level::Low).degrade();

        // Bluetooth Setup
        let bluetooth = Bluetooth::new(
            ctx.device.RADIO,
            ctx.device.TIMER0,
            &ctx.device.FICR,
            ctx.resources.ble_tx_buf,
            ctx.resources.ble_rx_buf,
            ctx.resources.tx_queue,
            ctx.resources.rx_queue,
        );

        ctx.spawn.boiler_measure_temperature().unwrap();

        let heater_config = HeaterConfig::new(96.0, 1.0, 0.0, 0.0);

        init::LateResources {
            boiler: Boiler::new(sensor_signal, sensor_vdd),
            bluetooth,
            boiler_timer: Timer::new(ctx.device.TIMER1),
            heater: Heater::new(heater_signal, heater_config),
        }
    }

    #[task(binds = RADIO, resources = [bluetooth], spawn = [bluetooth_worker], priority = 3)]
    fn bluetooth_radio(ctx: bluetooth_radio::Context) {
        if ctx.resources.bluetooth.handle_radio_interrupt() {
            ctx.spawn.bluetooth_worker().ok();
        }
    }

    #[task(binds = TIMER0, resources = [bluetooth], spawn = [bluetooth_worker], priority = 3)]
    fn bluetooth_timer(ctx: bluetooth_timer::Context) {
        if ctx.resources.bluetooth.handle_timer_interrupt() {
            ctx.spawn.bluetooth_worker().ok();
        }

        ctx.resources.bluetooth.drain_packet_queue();
    }

    #[task(resources = [bluetooth], priority = 2)]
    fn bluetooth_worker(mut ctx: bluetooth_worker::Context) {
        ctx.resources.bluetooth.lock(|bt| {
            bt.drain_packet_queue();
        });
    }

    #[task(schedule = [bluetooth_update_attrs])]
    fn bluetooth_update_attrs(ctx: bluetooth_update_attrs::Context) {
        // TODO: Fetch all resources and update the bluetooth attrs

        ctx.schedule
            .bluetooth_update_attrs(ctx.scheduled + ONE_SECOND.cycles())
            .unwrap();
    }

    #[task(resources = [bluetooth, boiler, boiler_timer, heater], priority = 2, schedule = [boiler_measure_temperature])]
    fn boiler_measure_temperature(ctx: boiler_measure_temperature::Context) {
        if let Ok(t) = ctx
            .resources
            .boiler
            .read_temperature(ctx.resources.boiler_timer)
        {
            let heater_on = ctx.resources.heater.control(t).ok().unwrap();
            defmt::info!("Temp is: {:f32}, Heater on: {:bool}", t, heater_on);
        } else {
            defmt::warn!("Temp read failed");
            // Turn the heater off until we get a good new reading for safety reasons.
            ctx.resources.heater.turn_heater_off().ok();
        }

        ctx.schedule
            .boiler_measure_temperature(ctx.scheduled + ONE_SECOND.cycles())
            .unwrap();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
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

#[panic_handler] // panicking behavior
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        defmt::error!(
            "panicked at {:str}:{:u32}:{:u32}",
            loc.file(),
            loc.line(),
            loc.column()
        )
    } else {
        // no location info
        defmt::error!("panicked")
    }

    exit()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
