//! Encapsulates the Bluetooth Low Energy Peripheral

use core::cmp;
use nrf52840_hal::pac::{FICR, RADIO, TIMER0};
use rubble::att::{AttUuid, Attribute, AttributeProvider, Handle, HandleRange};
use rubble::config::Config;
use rubble::l2cap::{BleChannelMap, L2CAPState};
use rubble::link::ad_structure::AdStructure;
use rubble::link::queue::{PacketQueue, SimpleQueue};
use rubble::link::{LinkLayer, Responder};
use rubble::security::NoSecurity;
use rubble::time::{Duration, Timer};
use rubble::uuid::Uuid16;
use rubble::Error;
use rubble_nrf5x::radio::{BleRadio, PacketBuffer};
use rubble_nrf5x::timer::BleTimer;
use rubble_nrf5x::utils::get_device_address;

pub enum BleConfig {}

impl Config for BleConfig {
    type Timer = BleTimer<TIMER0>;
    type Transmitter = BleRadio;
    type ChannelMapper = BleChannelMap<ControllerServiceAttrs<'static>, NoSecurity>;
    type PacketQueue = &'static mut SimpleQueue;
}

pub struct Bluetooth {
    link_layer: LinkLayer<BleConfig>,
    responder: Responder<BleConfig>,
    radio: BleRadio,
}

impl Bluetooth {
    pub fn new(
        radio: RADIO,
        timer: TIMER0,
        ficr: &FICR,
        tx_buf: &'static mut PacketBuffer,
        rx_buf: &'static mut PacketBuffer,
        tx_queue: &'static mut SimpleQueue,
        rx_queue: &'static mut SimpleQueue,
    ) -> Self {
        let ble_timer = BleTimer::init(timer);
        let device_address = get_device_address();

        let mut ble_radio = BleRadio::new(radio, ficr, tx_buf, rx_buf);

        let (tx, tx_cons) = tx_queue.split();
        let (rx_prod, rx) = rx_queue.split();

        // Create the actual BLE stack objects
        let mut ble_ll = LinkLayer::<BleConfig>::new(device_address, ble_timer);

        let ble_r = Responder::new(
            tx,
            rx,
            L2CAPState::new(BleChannelMap::with_attributes(ControllerServiceAttrs::new())),
        );

        // Send advertisement and set up regular interrupt
        let next_update = ble_ll
            .start_advertise(
                Duration::from_millis(200),
                &[AdStructure::CompleteLocalName("rusty-pid-controller")],
                &mut ble_radio,
                tx_cons,
                rx_prod,
            )
            .unwrap();

        ble_ll.timer().configure_interrupt(next_update);

        Self {
            radio: ble_radio,
            link_layer: ble_ll,
            responder: ble_r,
        }
    }

    pub fn link(&mut self) -> &mut LinkLayer<BleConfig> {
        &mut self.link_layer
    }

    pub fn responder(&mut self) -> &mut Responder<BleConfig> {
        &mut self.responder
    }

    pub fn radio(&mut self) -> &mut BleRadio {
        &mut self.radio
    }

    pub fn handle_radio_interrupt(&mut self) -> bool {
        if let Some(cmd) = self
            .radio
            .recv_interrupt(self.link_layer.timer().now(), &mut self.link_layer)
        {
            self.radio.configure_receiver(cmd.radio);
            self.link_layer.timer().configure_interrupt(cmd.next_update);

            return cmd.queued_work;
        }

        false
    }

    pub fn handle_timer_interrupt(&mut self) -> bool {
        if !self.link_layer.timer().is_interrupt_pending() {
            return false;
        }

        self.link_layer.timer().clear_interrupt();
        let cmd = self.link_layer.update_timer(&mut self.radio);
        self.radio.configure_receiver(cmd.radio);
        self.link_layer.timer().configure_interrupt(cmd.next_update);

        return cmd.queued_work;
    }

    pub fn drain_packet_queue(&mut self) {
        while self.responder.has_work() {
            self.responder.process_one().unwrap();
        }
    }
}

pub struct ControllerServiceAttrs<'a> {
    attributes: [Attribute<'a>; 3],
}

impl<'a> ControllerServiceAttrs<'a> {
    pub fn new() -> Self {
        Self {
            attributes: [
                Attribute::new(
                    Uuid16(0x2800).into(), // "Primary Service"
                    Handle::from_raw(0x0001),
                    &[0x00, 0x18], // "Generic Access" 0x1800
                ),
                Attribute::new(
                    Uuid16(0x2803).into(), // "Characteristic"
                    Handle::from_raw(0x0002),
                    &[
                        0x02, // 1 byte properties: READ = 0x02
                        0x03, 0x00, // 2 bytes handle = 0x0003
                        0x1F, 0x2A, // 2 bytes UUID = 0x2A1F (Temperature Celsius)
                    ],
                ),
                // Characteristic value (Temperature Celsius)
                Attribute::new(
                    AttUuid::Uuid16(Uuid16(0x2A1F)),
                    Handle::from_raw(0x0003),
                    &[0u8, 0u8],
                ),
            ],
        }
    }

    pub fn set_boiler_temp(&mut self, temp: &'a [u8]) {
        self.attributes[2].set_value(temp);
    }
}

impl<'a> AttributeProvider for ControllerServiceAttrs<'a> {
    fn for_attrs_in_range(
        &mut self,
        range: HandleRange,
        mut f: impl FnMut(&Self, Attribute<'_>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let count = self.attributes.len();
        let start = usize::from(range.start().as_u16() - 1); // handles start at 1, not 0
        let end = usize::from(range.end().as_u16() - 1);

        let attrs = if start >= count {
            &[]
        } else {
            let end = cmp::min(count - 1, end);
            &self.attributes[start..=end]
        };

        for attr in attrs {
            f(
                self,
                Attribute {
                    att_type: attr.att_type,
                    handle: attr.handle,
                    value: attr.value,
                },
            )?;
        }
        Ok(())
    }

    fn is_grouping_attr(&self, uuid: AttUuid) -> bool {
        uuid == Uuid16(0x2800) // FIXME not characteristics?
    }

    fn group_end(&self, handle: Handle) -> Option<&Attribute<'_>> {
        match handle.as_u16() {
            0x0001 => Some(&self.attributes[2]),
            0x0002 => Some(&self.attributes[2]),
            _ => None,
        }
    }
}
