use alloc::string::String;
pub fn vendor(id: u16) -> String {
    match id {
        0x1234 => return String::from("QEMU"),
        0x10EC => return String::from("REALTEK"),
        0x10DE => return String::from("NVIDIA"),
        0x15B6=> return String::from("IBM"),
        0x8086 => return String::from("INTEL"),
        _ => return String::from("UNKNOWN"),
    }
}

pub fn device(id: u16) -> String {
    match id {
        0x1111 => return String::from("VGA"),
        0x1237 => return String::from("82441FX CHIPSET"),
        0x8139 => return String::from("RTL8139"),
        0x7000 => return String::from("PIIX3 ISA"),
        0x7010 => return String::from("PIIX3 IDE"),
        0x7113 => return String::from("PIIX4 ACPI"),
        _ => return String::from("UNKNOWN"),
    }
}