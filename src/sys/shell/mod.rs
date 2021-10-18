use alloc::{string::String, vec::Vec};
use alloc::format;
use crate::arch::i386::cmos::CMOS;
use crate::{clear, print, println, sys::{self, mem}};
use crate::run;

mod cmd;

use super::keyboard;

pub type ShellProgram = fn(&Vec<&str>) -> usize;


pub fn start() {
    sys::keyboard::consume_char();
    println!("Cobalt Shell");

    loop {
        print!(">\r");
        let mut input_buf = &mut String::new();
        loop {
            sys::timer::pause(0.01);
            input(&mut input_buf);
            let exit_code = run(input_buf);
            if exit_code != 0 && exit_code < usize::MAX {
                println!("Program Returned With Exit Code {}", exit_code);
            };
            input_buf.clear();
        }
    }
}

pub fn run(command: &str) -> usize {
    let parts: Vec<&str> = command.split_ascii_whitespace().collect();
    if parts.len() == 0 {return usize::MAX;}
    let program_name: &str = &parts[0].to_ascii_lowercase();

    return match program_name {
        "mem" => {mem_stats(&parts)},
        "clear" => {clear!(); 0},
        "uptime" => {uptime(&parts)},
        "pause" => {pause(&parts)},
        "shutdown" => {shutdown(&parts)},
        "echo" => {echo(&parts)},
        "help" => {help(&parts)},
        "exit_test" => {test_exit(&parts)},
        "watch" => {watch(&parts)},
        "txt" => {cmd::text_editor::main(&parts)},
        "dsk" => {cmd::dsk::main(&parts)},
        "install" => {cmd::install::main(&parts)},
        "fs" => {cmd::fs::main(&parts)},
        "pci" => {cmd::pci::main(&parts)},
        "net" => {cmd::net::main(&parts)},
        "syscall" => {cmd::syscall::main(&parts)},
        _ => {
            println!("Unknown Command '{}'", program_name);
            usize::MAX
        }
    };
    
}


fn mem_stats(_args: &Vec<&str>) -> usize {
    println!("Mem Free: {} B", mem::free());
    println!("Mem Used: {} B", mem::used());
    println!("Mem Size: {} B", mem::size());
    return 0;
}

fn pause(args: &Vec<&str>) -> usize {
    let time: Option<&&str> = args.iter().nth(1);
    let time: &str = time.unwrap_or(&"0");
    let time: f64 = time.parse().expect("Expected A Number");

    sys::timer::pause(time);
    return 0;
}

fn uptime(_args: &Vec<&str>) -> usize {
    println!("System Uptime: {:0.3} seconds" , sys::timer::uptime_seconds());
    let rtc = CMOS::new().rtc();
    println!("Current Time: {}:{}:{}",rtc.hour, rtc.minute, rtc.second );
    return 0;
}

fn shutdown(_args: &Vec<&str>) -> usize {
    println!("Are You Sure You Want To Shutdown? y/N");
    let mut input_buffer = &mut String::new();
    input(&mut input_buffer);
    if input_buffer.starts_with("y") {
        println!("Shutting Down in 5 Seconds!");
        run("pause 5");
        sys::shutdown();

    } else {
        println!("Shutdown Stopped!");
        1
    }
}

fn echo(args: &Vec<&str>) -> usize {
    let mut msg = String::new();
    for arg in args.iter().skip(1) {
        msg.push_str(arg);
        msg.push(' ');
    }
    msg.pop();

    let escaped_msg = msg.replace("\\n", "\n");

    println!("{}", escaped_msg);

    return 0;
}

fn input(buffer: &mut String) {
    loop {
        print!(">   \r");
        if let Some(key) = sys::keyboard::consume_char() {
            if key == '\x7F' || key == '\x08' {
                buffer.pop();
            } else if key == '\n' {
                print!("> {}  \n", buffer);
                return;
            } else {
                buffer.push(key);
            }
        }
        print!("> {}  \r", buffer);
        run("pause 0.01");
    }
}

fn help(_: &Vec<&str>) -> usize {
    run!("clear");
    run!("Echo CobaltOS Shell Version 1.0.");
    run!("Echo Built In Commands: ");
    run!("Echo 1. mem - View Heap Usage.");
    run!("Echo 2. pause <seconds> - Halt Execution for <seconds>.");
    run!("Echo 3. uptime - prints the system uptime in seconds");
    run!("Echo 4. shutdown - shuts the system down, only works on Qemu, requires input.");
    run!("Echo 5. dsk - Various Disk Utilities");
    run!("Echo 6. echo - Echos back the arguments to the screen");
    run!("Echo 7. install - Copies The contents of Drive 0:0 To Drive 0:1, doesn't ask for authentication.");
    return 0;
}

fn watch(args: &Vec<&str>) -> usize {
    if args.len() < 3 {println!("Usage watch <command> <poll rate (seconds)>"); return 1;};
    loop {
        if run!("exit_test") == 1 {
            return 0;
        } else {
            run!("clear");
            run!("echo {}", "Press [Escape] To Exit.");
            run!("{}", args[1]);
        }
        run!("pause {}", args[2]);
    }
}

fn test_exit(_args: &Vec<&str>) -> usize {
    if let Some(keycode) = keyboard::consume_char() {
        if keycode == '\x1b' {
            return 1;
        } else {
            return 0;
        }
    };
    return 0;
}



#[macro_export]
macro_rules! run {
    ($($arg:tt)*) => {
        {
            use alloc::format;
            let command_str = format!($($arg)*);
            $crate::sys::shell::run(&command_str)
        }
    };
}