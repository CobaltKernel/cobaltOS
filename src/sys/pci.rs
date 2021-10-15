use x86_64::instructions::port::Port;
use bit_field::BitField;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::vec::Vec;

use crate::println;

const PCI_CONFIG_BASE: u32 = 0x8000_0000;
const PCI_ADDR_PORT: u16 = 0xCF8;
const PCI_DATA_PORT: u16 = 0xCFC;

lazy_static! {
    pub static ref PCI_DEVICES: Mutex<Vec<DeviceConfig>> = Mutex::new(Vec::new());
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceConfig {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub status: u16,
    pub command: u16,
    pub base_addresses: [u32; 6],
    pub interrupt_pin: u8,
    pub interrupt_line: u8,
}

impl DeviceConfig {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        let vendor_id = get_vendor_id(bus, device, function);
        let device_id = get_device_id(bus, device, function);

        let mut register = ConfigRegister::new(bus, device, function, 0x04);
        let data = register.read();
        let command = data.get_bits(0..16) as u16;
        let status = data.get_bits(16..32) as u16;

        let mut register = ConfigRegister::new(bus, device, function, 0x3C);
        let data = register.read();
        let interrupt_line = data.get_bits(0..8) as u8;
        let interrupt_pin = data.get_bits(8..16) as u8;

        let mut base_addresses: [u32; 6] = [0; 6];
        for i in 0..6 {
            let offset = 0x10 + ((i as u8) << 2);
            let mut register = ConfigRegister::new(bus, device, function, offset);
            base_addresses[i] = register.read();
        };

        Self {
            bus,
            device,
            function,

            // Configuration Space registers
            vendor_id,
            device_id,
            status,
            command,
            base_addresses,
            interrupt_pin,
            interrupt_line,
        }

    }

    pub fn enable_bus_mastering(&mut self) {
        let mut register = ConfigRegister::new(self.bus, self.device, self.function, 0x04);
        let mut data = register.read();
        data.set_bit(2, true);
        register.write(data);
    }
}

fn get_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    let mut register = ConfigRegister::new(bus, device, function, 0x00);
    register.read().get_bits(0..16) as u16
}

fn get_device_id(bus: u8, device: u8, function: u8) -> u16 {
    let mut register = ConfigRegister::new(bus, device, function, 0x00);
    register.read().get_bits(16..32) as u16
}

fn get_header_type(bus: u8, device: u8, function: u8) -> u8 {
    let mut register = ConfigRegister::new(bus, device, function, 0x0C);
    register.read().get_bits(16..24) as u8
}

pub fn init() {
    for bus in 0..256 {
        check_bus(bus as u8);
    }
}

fn check_bus(bus: u8) {
    for device in 0..32 {
        check_device(bus, device);
    }
}

fn check_device(bus: u8, device: u8) {
    let function = 0;

    let vendor_id = get_vendor_id(bus, device, function);
    if vendor_id == 0xFFFF {
        return; // Device doesn't exist
    }
    add_device(bus, device, function);

    // Multi-function devices
    let header_type = get_header_type(bus, device, function);
    if header_type & 0x80 != 0 {
        for function in 1..8 {
            let vendor_id = get_vendor_id(bus, device, function);
            if vendor_id != 0xFFFF {
                add_device(bus, device, function);
            }
        }
    }
}

fn add_device(bus: u8, device: u8, function: u8) {
    let config = DeviceConfig::new(bus, device, function);
    PCI_DEVICES.lock().push(config);
    println!("PCI {:04}:{:02}:{:02} [{:04X}:{:04X}]...", bus, device, function, config.vendor_id, config.device_id);
}

pub fn find_device(vendor: u16, device_id: u16) -> Option<DeviceConfig> {
    let devices = &*PCI_DEVICES.lock();
    for device in devices {
        if device.device_id == device_id && device.vendor_id == vendor {
            return Some(*device)
        };
    }
    return None;
}

struct ConfigRegister {
    addr_port: Port<u32>,
    data_port: Port<u32>,
    addr: u32,
}

impl ConfigRegister {
    pub fn new(bus: u8, device: u8, function: u8, offset: u8) -> Self {
        Self {
            addr_port: Port::new(PCI_ADDR_PORT),
            data_port: Port::new(PCI_DATA_PORT),
            addr: PCI_CONFIG_BASE |
                ((bus as u32) << 16) |
                ((device as u32) << 11) |
                ((function as u32) << 8) |
                ((offset as u32) & 0xFC),
        }
    }

    pub fn read(&mut self) -> u32 {
        unsafe {
            self.addr_port.write(self.addr);
            self.data_port.read()
        }
    }

    pub fn write(&mut self, data: u32) {
        unsafe {
            self.addr_port.write(self.addr);
            self.data_port.write(data);
        }
    }
}
