//! Experimental code for the touch screen display.

#![no_main]
#![cfg_attr(not(test), no_std)]

#[allow(unused_imports)]
use defmt_rtt as _;
use display_interface::DataFormat::U16BEIter;
use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterface;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::RawData;
use embedded_hal::{blocking::delay::DelayMs, spi::MODE_0};
use ili9341::Ili9341;
#[allow(unused_imports)]
use nrf52840_hal as _;
use nrf52840_hal::{
    gpio::{Level, Output, Pin, PushPull},
    pac::SPIM0,
    spim::{Frequency, Pins},
    Spim,
};
use nrf52840_hal::{prelude::OutputPin, Timer};

use core::iter::once;
use core::sync::atomic::{AtomicUsize, Ordering};
use display_interface::DataFormat::U8Iter;
use groundhog_nrf52::GlobalRollingTimer;
use nrf52840_hal::gpio::p0;
use panic_probe as _;

#[rtic::app(device = nrf52840_hal::pac, peripherals = true, monotonic = groundhog_nrf52::GlobalRollingTimer)]
const APP: () = {
    struct Resources {
        display: Ili9341<
            SPIInterface<Spim<SPIM0>, Pin<Output<PushPull>>, Pin<Output<PushPull>>>,
            Pin<Output<PushPull>>,
        >,
    }

    #[init(spawn = [display_render])]
    fn init(ctx: init::Context) -> init::LateResources {
        GlobalRollingTimer::init(ctx.device.TIMER0);

        let port0 = p0::Parts::new(ctx.device.P0);

        // Display Pins

        // MI 0.15
        let _miso = port0.p0_15.into_floating_input().degrade();
        // MO 0.13
        let mosi = port0.p0_13.into_push_pull_output(Level::Low).degrade();
        // SCK 0.14
        let sck = port0.p0_14.into_push_pull_output(Level::Low).degrade();
        // A5 CS 0.03
        let cs = port0.p0_03.into_push_pull_output(Level::Low).degrade();
        // A4 D/C 0.02
        let dc = port0.p0_02.into_push_pull_output(Level::Low).degrade();
        // A3 Weiss 0.28
        let rst = port0.p0_28.into_push_pull_output(Level::Low).degrade();

        let frequency = Frequency::M16;
        let spi_pins = Pins {
            sck,
            mosi: Some(mosi),
            miso: None,
        };
        let spi = Spim::new(ctx.device.SPIM0, spi_pins, frequency, MODE_0, 0);
        let spi_interface = SPIInterface::new(spi, dc, cs);
        let mut display_timer = Timer::new(ctx.device.TIMER1);

        let display = Ili9341::new(spi_interface, rst, &mut display_timer).unwrap();
        //let mut display = Display::new(spi_interface, rst);
        //display.init(&mut display_timer);

        ctx.spawn.display_render().unwrap();

        init::LateResources { display }
    }

    #[task(resources = [display], schedule = [display_render])]
    fn display_render(ctx: display_render::Context) {
        defmt::info!("Rendering Display");

        ctx.resources
            .display
            .draw_raw(0, 0, 10, 10, &[RawU16::from(0x780F).into_inner()])
            .ok();

        //ctx.resources.display.invert(true);

        ctx.schedule
            .display_render(ctx.scheduled + 1_000_000)
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
    cortex_m::asm::udf()
}

pub struct Display<IFACE> {
    interface: IFACE,
    rst: Pin<Output<PushPull>>,
}

impl<IFACE> Display<IFACE>
where
    IFACE: WriteOnlyDataCommand,
{
    pub fn new(interface: IFACE, rst: Pin<Output<PushPull>>) -> Self {
        Self { interface, rst }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), DisplayError>
    where
        DELAY: DelayMs<u16>,
    {
        // Do hardware reset by holding reset low for at least 10us
        self.rst.set_low().map_err(|_| DisplayError::OutputPin)?;
        delay.delay_ms(1);
        // Set high for normal operation
        self.rst.set_high().map_err(|_| DisplayError::OutputPin)?;

        // Wait 5ms after reset before sending commands
        // and 120ms before sending Sleep Out
        delay.delay_ms(5);

        // Do software reset
        self.command(DisplayCommand::SoftwareReset, &[])?;

        delay.delay_ms(150);

        self.command(DisplayCommand::UndocumentedInit, &[0x03, 0x80, 0x02])?;
        self.command(DisplayCommand::PowerControlB, &[0x00, 0xC1, 0x30])?;
        self.command(DisplayCommand::PowerOnSeqControl, &[0x64, 0x03, 0x12, 0x81])?;
        self.command(DisplayCommand::DriverTimingControlA, &[0x85, 0x00, 0x78])?;

        self.command(
            DisplayCommand::PowerControlA,
            &[0x39, 0x2C, 0x00, 0x34, 0x02],
        )?;
        self.command(DisplayCommand::PumpRatioControl, &[0x20])?;
        self.command(DisplayCommand::DriverTimingControlB, &[0x00, 0x00])?;

        self.command(DisplayCommand::PowerControl1, &[0x23])?;
        self.command(DisplayCommand::PowerControl2, &[0x10])?;
        self.command(DisplayCommand::VcomControl1, &[0x3e, 0x28])?;
        self.command(DisplayCommand::VcomControl2, &[0x86])?;

        self.command(DisplayCommand::MemoryAccessControl, &[0x48])?; // Portrait
        self.command(DisplayCommand::VerticalScrollingStartAddress, &[0x00])?;
        self.command(DisplayCommand::PixelFormatSet, &[0x55])?;
        self.command(DisplayCommand::FrameRateControl, &[0x00, 0x18])?;

        self.command(DisplayCommand::DisplayFunctionControl, &[0x08, 0x82, 0x27])?;
        self.command(DisplayCommand::Enable3G, &[0x00])?;
        self.command(DisplayCommand::GammaSet, &[0x01])?;
        self.command(
            DisplayCommand::PositiveGammaCorrection,
            &[
                0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09,
                0x00,
            ],
        )?;
        self.command(
            DisplayCommand::NegativeGammaCorrection,
            &[
                0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36,
                0x0F,
            ],
        )?;

        self.command(DisplayCommand::SleepOut, &[])?;
        delay.delay_ms(150);
        self.command(DisplayCommand::DisplayOn, &[])?;
        delay.delay_ms(150);

        Ok(())
    }

    fn command(&mut self, cmd: DisplayCommand, args: &[u8]) -> Result<(), DisplayError> {
        self.interface
            .send_commands(U8Iter(&mut once(cmd as u8)))
            .map_err(|_| DisplayError::Interface)?;
        self.interface
            .send_data(U8Iter(&mut args.iter().cloned()))
            .map_err(|_| DisplayError::Interface)
    }

    pub fn invert(&mut self, invert: bool) -> Result<(), DisplayError> {
        if invert {
            self.command(DisplayCommand::InvertOn, &[])
        } else {
            self.command(DisplayCommand::InvertOff, &[])
        }
    }

    pub fn draw_raw(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        data: &[u16],
    ) -> Result<(), DisplayError> {
        self.set_window(x0, y0, x1, y1)?;
        self.write_iter(data.iter().cloned())
    }

    fn write_iter<I: IntoIterator<Item = u16>>(&mut self, data: I) -> Result<(), DisplayError> {
        self.command(DisplayCommand::MemoryWrite, &[])?;
        self.interface
            .send_data(U16BEIter(&mut data.into_iter()))
            .map_err(|_| DisplayError::Interface)
    }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), DisplayError> {
        self.command(
            DisplayCommand::ColumnAddressSet,
            &[
                (x0 >> 8) as u8,
                (x0 & 0xff) as u8,
                (x1 >> 8) as u8,
                (x1 & 0xff) as u8,
            ],
        )?;
        self.command(
            DisplayCommand::PageAddressSet,
            &[
                (y0 >> 8) as u8,
                (y0 & 0xff) as u8,
                (y1 >> 8) as u8,
                (y1 & 0xff) as u8,
            ],
        )?;
        Ok(())
    }
}

pub enum DisplayError {
    OutputPin,
    Interface,
}

#[derive(Clone, Copy)]
enum DisplayCommand {
    UndocumentedInit = 0xEF,
    PowerControlB = 0xCF,
    SoftwareReset = 0x01,
    ColumnAddressSet = 0x2A,
    PageAddressSet = 0x2B,
    MemoryWrite = 0x2C,
    PowerOnSeqControl = 0xED,
    DriverTimingControlA = 0xE8,
    PowerControlA = 0xCB,
    PumpRatioControl = 0xF7,
    DriverTimingControlB = 0xEA,
    PowerControl1 = 0xC0,
    PowerControl2 = 0xC1,
    VcomControl1 = 0xC5,
    VcomControl2 = 0xC7,
    MemoryAccessControl = 0x36,
    VerticalScrollingStartAddress = 0x37,
    PixelFormatSet = 0x3A,
    FrameRateControl = 0xB1,
    DisplayFunctionControl = 0xB6,
    Enable3G = 0xF2,
    GammaSet = 0x26,
    PositiveGammaCorrection = 0xE0,
    NegativeGammaCorrection = 0xE1,
    SleepOut = 0x11,
    DisplayOn = 0x29,
    _DisplayOff = 0x28,
    InvertOn = 0x21,
    InvertOff = 0x20,
}
