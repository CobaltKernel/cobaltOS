use lazy_static::lazy_static;

lazy_static! {
    pub static ref VENDORS: [&'static str; 65536] = {
        let mut vendors = ["UNK"; 65536];
        vendors[0x0001] = "SAFENET INC";
        vendors[0x10DE] = "NVIDIA";
        vendors[0x15B6] = "IBM";
        vendors[0x8086] = "INTEL";
        vendors
    };
}