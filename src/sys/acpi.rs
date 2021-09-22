use core::ptr::NonNull;

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use alloc::boxed::Box;
use aml::{AmlContext, AmlName, AmlValue, Handler};
use x86_64::{PhysAddr, instructions::port::Port};
use crate::{serial_println, sys};

#[allow(dead_code)]
#[repr(u64)]
enum FADT {
    SciInterrupt     = 46, // u16,
    SmiCmdPort       = 48, // u32,
    AcpiEnable       = 52, // u8,
    AcpiDisable      = 53, // u8,
    S4biosReq        = 54, // u8,
    PstateControl    = 55, // u8,
    Pm1aEventBlock   = 56, // u32,
    Pm1bEventBlock   = 60, // u32,
    Pm1aControlBlock = 64, // u32,
    Pm1bControlBlock = 68, // u32,
}

fn read_addr<T>(addr: usize) -> T where T: Copy {
    let virtual_addr = sys::mem::phys_to_virt(PhysAddr::new(addr as u64));
    unsafe {*virtual_addr.as_ptr::<T>()}
} 

fn read_fadt<T>(addr: usize, offset: FADT) -> T where T: Copy {
    read_addr(addr + offset as usize)
}

pub fn shutdown() -> ! {
    let mut pm1a_control_block = 0;
    let mut sleep_type = 0;
    let sleep_len = 1 << 13;

    serial_println!("ACPI Shutdown...");
    let mut aml = AmlContext::new(Box::new(CobaltAmlHandler), aml::DebugVerbosity::None);
    let mut res = unsafe { AcpiTables::search_for_rsdp_bios(CobaltAcpiHandler)};
    match res {
        Ok(acpi) => {
            for (id, sdt) in acpi.sdts {
                if id.as_str() == "FACP" {
                    pm1a_control_block = read_fadt(sdt.physical_address, FADT::Pm1aControlBlock);
                }
            }

            match &acpi.dsdt {
                Some(dsdt) => {
                    let address = sys::mem::phys_to_virt(PhysAddr::new(dsdt.address as u64));
                    let stream = unsafe { core::slice::from_raw_parts(address.as_ptr(), dsdt.length as usize) };
                    if aml.parse_table(stream).is_ok() {
                        let name = AmlName::from_str("\\_S5").unwrap();
                        if let Ok(AmlValue::Package(s5)) = aml.namespace.get_by_path(&name) {
                            if let AmlValue::Integer(value) = s5[0] {
                                sleep_type = value;
                            }

                        } else {
                            serial_println!("Failed To Parse AML in DSDT.")
                        }
                    };
                },

                None => {},
            }
        },

        Err(_e) => {
            serial_println!("Unable To Locate RSDP.");
        }

        
    }
    let mut port: Port<u16> = Port::new(pm1a_control_block as u16);
    unsafe {
        port.write((sleep_len | sleep_type) as u16);
    };

    sys::halt();
}



#[derive(Clone)]
pub struct CobaltAcpiHandler;

impl AcpiHandler for CobaltAcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virtual_address = sys::mem::phys_to_virt(PhysAddr::new(physical_address as u64));
        PhysicalMapping::new(physical_address, NonNull::new(virtual_address.as_mut_ptr()).unwrap(), size, size, Self)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

struct CobaltAmlHandler;

impl Handler for CobaltAmlHandler {
    fn read_u8(&self, address: usize) -> u8 { read_addr::<u8>(address) }
    fn read_u16(&self, address: usize) -> u16 { read_addr::<u16>(address) }
    fn read_u32(&self, address: usize) -> u32 { read_addr::<u32>(address) }
    fn read_u64(&self, address: usize) -> u64 { read_addr::<u64>(address) }
    fn write_u8(&mut self, _address: usize, _value: u8) { unimplemented!() }
    fn write_u16(&mut self, _address: usize, _value: u16) { unimplemented!() }
    fn write_u32(&mut self, _address: usize, _value: u32) { unimplemented!() }
    fn write_u64(&mut self, _address: usize, _value: u64) { unimplemented!() }
    fn read_io_u8(&self, _port: u16) -> u8 { unimplemented!() }
    fn read_io_u16(&self, _port: u16) -> u16 { unimplemented!() }
    fn read_io_u32(&self, _port: u16) -> u32 { unimplemented!() }
    fn write_io_u8(&self, _port: u16, _value: u8) { unimplemented!() }
    fn write_io_u16(&self, _port: u16, _value: u16) { unimplemented!() }
    fn write_io_u32(&self, _port: u16, _value: u32) { unimplemented!() }
    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 { unimplemented!() }
    fn read_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u16 { unimplemented!() }
    fn read_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u32 { unimplemented!() }
    fn write_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u8) { unimplemented!() }
    fn write_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u16) { unimplemented!() }
    fn write_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u32) { unimplemented!() }
}