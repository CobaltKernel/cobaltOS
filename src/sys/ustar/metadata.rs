use alloc::string::String;
use alloc::borrow::ToOwned;
use core::mem::size_of;
use alloc::vec::Vec;
use alloc::vec;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Metadata {

    addr: u32,

    file_name: String, // Offset:   0, Size: 100
    file_mode: String, // Offset: 100, Size:   8
    owner_id:  u32,    // Offset: 108, Size:   8
    group_id:  u32,    // Offset: 116, Size:   8
    size:      u64,    // Offset: 124, Size:  12
    modi_time: u64,    // Offset: 136, Size:  12
    chksum:    String, // Offset: 148, Size:   8
    type_flag: u8,     // Offset: 156, Size:   1
    link_name: String, // Offset: 157, Size: 100
    magic:     String, // Offset: 257, Size:   6
    version:   String, // Offset: 263, Size:   2
    uname:     String, // Offset: 265, Size:  32
    gname:     String, // Offset: 297, Size:  32
    dev_maj:   u32,    // Offset: 329, Size:   8
    dev_min:   u32,    // Offset: 337, Size:   8
    prefix:    String, // Offset: 345, Size: 155
}

impl Metadata {
    pub fn create_file(addr: u32, name: &str, data: &[u8]) -> Self {
        let mut meta = Self {
            addr,
            file_name: name.to_owned(),
            chksum: String::new(),
            owner_id: 0,
            group_id: 0,
            gname: String::from("gvenn"),
            uname: String::from("gvenn"),
            dev_maj: 0,
            dev_min: 0,
            magic: "USTAR\0".to_owned(),
            version: "00".to_owned(),
            file_mode: "00777".to_owned(),
            size: data.len() as u64, 
            modi_time: 0,
            type_flag: b'0',
            link_name: "".to_owned(),
            prefix: "".to_owned(),
        };

        meta.calc_checksum();
        meta
    }

    pub fn calc_checksum(&mut self) {
        self.chksum = "        ".to_owned();

        let mut check: u32 = 0;

        for i in 0..size_of::<Self>() {
            check += unsafe {*(self.raw().offset(i as isize))} as u32;
        }

        self.chksum = alloc::format!("{:06o}  ", check);

        unsafe { self.chksum.as_bytes_mut()[6] = b'\0' }
        unsafe { self.chksum.as_bytes_mut()[7] = b' ' }
    }

    unsafe fn raw(&self) -> *const u8 {
        (self as *const Self) as *const u8
    }

    pub fn from(addr: u32, data: &[u8]) -> Self {
        let mut name: Vec<u8> = Vec::new();
        for index in 0..100 {
            if data[index] != 0 {
                name.push(data[index]);
            }
        }

        let name: String = String::from_utf8(name).expect("Failed To Parse Name");

        

        let mut size: Vec<u8> = Vec::new(); 
        for index in 124..136 {
            if data[index] != 0 {
                size.push(data[index]);
            }
        }


        
        let size: String = String::from_utf8(size).expect("Failed To Parse Size");

        let mut meta = Self {
            addr,
            file_name: name,
            chksum: String::new(),
            owner_id: 0,
            group_id: 0,
            gname: String::from("gvenn"),
            uname: String::from("gvenn"),
            dev_maj: 0,
            dev_min: 0,
            magic: "USTAR\0".to_owned(),
            version: "00".to_owned(),
            file_mode: "00777".to_owned(),
            size: u64::from_str_radix(&size, 8).unwrap_or(0), 
            modi_time: 0,
            type_flag: b'0',
            link_name: "".to_owned(),
            prefix: "".to_owned(),
        };

        meta
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn block_length(&self) -> usize {
        if self.size == 0 {return 1};
        let mut blocks = self.size / 512;
        if self.size % 512 != 0 {blocks += 1;};

        (blocks + 1) as usize
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn set_file_name(&mut self, name: &str) {
        self.file_name = String::from(name);
    }
    
}



