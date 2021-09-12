
use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable, PageFaultErrorCode};
use crate::{serial, serial_print, serial_println, sys::{self, keyboard}};
use super::{gdt, *};
use crate::interrupts::pics::*;

use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Cascade,
    Com1,
    Com2,
    Lpt2,
    FloppyDisk,
    Lpt1,
    CmosRtc,
    Free1,
    Free2,
    Free3,
    Ps2Mouse,
    Fpu,
    PrimaryAta,
    SecondaryAta,

    SystemCalls = 80
}



impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(on_breakpoint);
		unsafe {
			idt.double_fault.set_handler_fn(on_double_fault)
			.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
		}
		idt.page_fault.set_handler_fn(on_page_fault);

		idt[InterruptIndex::Timer.as_usize()].set_handler_fn(on_timer_tick); 		
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(on_key);
        idt[InterruptIndex::Com1.as_usize()].set_handler_fn(on_com1_ready);
        idt[InterruptIndex::Lpt1.as_usize()].set_handler_fn(on_spurious_irq);
        idt[InterruptIndex::PrimaryAta.as_usize()].set_handler_fn(on_ata_bus0_rdy);
        idt[InterruptIndex::SecondaryAta.as_usize()].set_handler_fn(on_ata_bus1_rdy);
        idt
    };
}


pub fn init() {
	IDT.load();
}

extern "x86-interrupt" fn on_breakpoint(_: InterruptStackFrame) {
	serial_println!("Breakpoint Hit!");
} 

extern "x86-interrupt" fn on_double_fault(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}


extern "x86-interrupt" fn on_page_fault(stack_frame: InterruptStackFrame, ec: PageFaultErrorCode) {
	serial_println!("Page Fault: {:?},  Addr: {:?}",ec,x86_64::registers::control::Cr2::read());
}

extern "x86-interrupt" fn on_timer_tick(
    _stack_frame: InterruptStackFrame)
{
    crate::sys::timer::increment();
	send_eoi(InterruptIndex::Timer.as_u8());
}

extern "x86-interrupt" fn on_com1_ready(_: InterruptStackFrame) {
    serial_print!("Serial Line: {}", serial::read());
    send_eoi(InterruptIndex::Com1.as_u8());
}

extern "x86-interrupt" fn on_key(_: InterruptStackFrame)
{
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            keyboard::set_keycode(key);
        }
    }
	send_eoi(InterruptIndex::Keyboard.as_u8());
}

extern "x86-interrupt" fn on_spurious_irq(_: InterruptStackFrame) {
    if !pics::is_spurious(InterruptIndex::Lpt1.as_u8()) {
        send_eoi(InterruptIndex::Lpt1.as_u8());
    }
}



extern "x86-interrupt" fn on_ata_bus0_rdy(_: InterruptStackFrame) {
	send_eoi(14);
}

extern "x86-interrupt" fn on_ata_bus1_rdy(_: InterruptStackFrame) {
    send_eoi(15);
}


extern "x86-interrupt" fn default(_: InterruptStackFrame) {
    
}