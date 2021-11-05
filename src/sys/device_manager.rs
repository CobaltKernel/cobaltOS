use alloc::vec::Vec;
use super::{pci::{*, self}, pci_details, storage::fs::dev_handle::{AtaDevice, DeviceHandle, MemDevice}};

pub enum Device {
    PCIDev(DeviceConfig),
    BlockDev(DeviceHandle),
}

impl Device {
    pub fn block_dev(&self) -> Option<&DeviceHandle> {
        match self {
            Self::PCIDev(_) => {None},
            Self::BlockDev(handle) => Some(handle)
        }
    }

    pub fn pci_dev(&self) -> Option<&DeviceConfig> {
        match self {
            Self::PCIDev(handler) => Some(handler),
            Self::BlockDev(_) => None,
        }
    }
}

/// Converts A Path Formatted in <Device Type>/ID
/// Example:
///     ATA/0/0
///     MEM/B8000/A0000
///     PCI/REALTEK/RTL8139
pub fn get_device(path: &str) -> Option<Device> {

    path.replace(":", "/");

    let sections: Vec<&str> = path.split("/").collect();
    match sections[0] {
        "PCI" => build_pci(&sections),
        "ATA" => build_ata(&sections),
        "MEM" => build_mem(),
        _ => None,
    }
}

/// id[0] = (ignored)
/// id[1] = Vendor
/// id[2] = device
fn build_pci(id: &Vec<&str>) -> Option<Device> {
    let vendor_id = pci_details::from_vendor_str(id[1]);
    let device_id = pci_details::from_dev_str(id[2]);
    if let Some(device) = pci::find_device(vendor_id, device_id) {
        return Some(Device::PCIDev(device));
    } else {
        return None;
    }
}

fn build_mem() -> Option<Device> {
    return Some(Device::BlockDev(DeviceHandle::MemBlockDevice(MemDevice::new())))
}
/// id[0] => (Ignored)
/// id[1] => Bus Index
/// id[2] => Drive Index
fn build_ata(id: &Vec<&str>) -> Option<Device> {
    let bus: u8 = id[1].parse().expect("Failed To Parse Value");
    let drive: u8 = id[2].parse().expect("Failed To Parse Value");
    return Some(Device::BlockDev(DeviceHandle::AtaBlockDevice(AtaDevice::new(bus, drive))))
}


#[test_case]
pub fn test_pci() {
    assert!(get_device("PCI/REALTEK/RTL8139").is_some());
}

