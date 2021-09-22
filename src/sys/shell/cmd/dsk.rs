

use alloc::vec::Vec;

use crate::{print, println, serial_print, serial_println, sys::{ata, mem, shell::run}};

pub fn main(args: &Vec<&str>) -> usize {
    if args.len() < 2 {
        println!("Usage dsk <subcommand>");
        return 1;
    };
    if args[1] == "ls" {
        return ls(args);
    }

    if args[1] == "read" {
        if args.len() < 5 { println!("Usage dsk read <bus> <drive> <block>"); return 2 };
        return read(args);
    }

    if args[1] == "format" {
        if args.len() < 4 { println!("Usage dsk write <bus> <drive>"); return 2 };
        return format(args);
    }

    if args[1] == "copy" {
        if args.len() < 6 { println!("Usage dsk copy <bus src> <drive src> <bus dest> <drive dest>"); return 2 };
        return copy(args);
    }
    1
}

fn ls(args: &Vec<&str>) -> usize {
    let disks = ata::list();
    for disk in disks {
        println!("Disk {}:{} - Model: {} - Serial: {}, Size: {} {}", disk.0, disk.1, disk.2, disk.3, disk.4, disk.5);
    }
    0
}

fn read(args: &Vec<&str>) -> usize {
    let mut buffer: [u8; 512] = [0; 512];
    let bus: u8 = args[2].parse().expect("Yeets");
    let drive: u8 = args[3].parse().expect("Yeets");
    let block: u32 = args[4].parse().expect("Yeets");

    ata::read(bus, drive, block, &mut buffer);

    for row in (0..buffer.len()).step_by(16) {
        serial_print!("{:03x}: ", row);
        for col in 0..16 {
            let index = row + col;
                print!("{:02x} ", buffer[index]);
                serial_print!("{:02x} ", buffer[index]);
        }
        println!();
        serial_println!();
    }

    0
}

pub fn format(args: &Vec<&str>) -> usize {
    let bus: u8 = args[2].parse().expect("Yeets");
    let drive: u8 = args[3].parse().expect("Yeets");
    let sectors = ata::sector_count(bus, drive);
    for block in 0..sectors {
<<<<<<< HEAD
        print!("Formatting Block {:04}/{:04} {:04} MB...\r", block, sectors, (block * 512) / mem::MB as u32);
        ata::write(bus, drive, block, &[0; 512]);
    }
    print!("Formatting Block {:04}/{:04}...\n", sectors, sectors);
=======
        print!("Formatting Block {:04}/{:04} Of Drive {}:{} {:04} MB ...\r", block, sectors, bus, drive, (block * 512) / mem::MB as u32);
        ata::write(bus, drive, block, &[0; 512]);
    }
    print!("Formatting Block {:04}/{:04} Of Drive {}:{} {:04} MB ...\n", sectors, sectors, bus, drive, (sectors * 512) / mem::MB as u32);
>>>>>>> 2d36125 (Removed test.img)
    0
}

pub fn copy(args: &Vec<&str>) -> usize { 
    let bus_src: u8 = args[2].parse().expect("Yeets");
    let drive_src: u8 = args[3].parse().expect("Yeets");

    let source_sectors = ata::sector_count(bus_src, drive_src);

    let bus_dest: u8 = args[4].parse().expect("Yeets");
    let drive_dest: u8 = args[5].parse().expect("Yeets");

    let dest_sectors = ata::sector_count(bus_dest, drive_dest);
<<<<<<< HEAD

    run!("dsk format {} {}", bus_dest, drive_dest);
    if (source_sectors > dest_sectors) {return 3};
    
    for block in 0..source_sectors {
        print!("Formatting Block {:04}/{:04}...\r", block, source_sectors);
=======
    if source_sectors > dest_sectors {
        run!("echo Disk Is Not Big Enough! ");
        return 3;
    };

    run!("dsk format {} {}", bus_dest, drive_dest);
    
    for block in 0..source_sectors {
        print!("Copying Block {:04}/{:04}...\r", block, source_sectors);
>>>>>>> 2d36125 (Removed test.img)
        let mut buffer: [u8; 512] = [0; 512];
        ata::read(bus_src, drive_src, block, &mut buffer);
        ata::write(bus_dest, drive_dest, block, &buffer);
    }
<<<<<<< HEAD
    print!("Formatting Block {:04}/{:04}...\r", source_sectors, source_sectors);
=======
    print!("Copying Block {:04}/{:04}...\n", source_sectors, source_sectors);
>>>>>>> 2d36125 (Removed test.img)

    0
}