
use spin::Mutex;
use x86_64::{instructions::{interrupts, port::Port}, structures::idt::{InterruptStackFrame, InterruptDescriptorTable, PageFaultErrorCode}};
use crate::{arch::i386::syscalls, debug, inb, println, serial, serial_print, serial_println, sys::{self, keyboard}};
use super::{gdt, pics::{PIC_1_OFFSET, send_eoi}};
use super::pics;


use lazy_static::lazy_static;


const PIC1: u16 = 0x21;
const PIC2: u16 = 0xA1;

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


#[allow(dead_code)]
fn interrupt_index(irq: u8) -> u8 {
    super::pics::PIC_1_OFFSET + irq
}

#[allow(unused)]
macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq]();
            unsafe { sys::pic::PICS.lock().notify_end_of_interrupt(interrupt_index($irq)); }
        }
    };
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

    pub static ref IRQ_HANDLERS: Mutex<[fn(); u8::MAX as usize]> = Mutex::new([default_irq_handler; u8::MAX as usize]);
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

        unsafe {
        idt[0x80].
                set_handler_fn(core::mem::transmute(wrap_syscall as *mut fn())).
                set_stack_index(0).
                set_privilege_level(x86_64::PrivilegeLevel::Ring0);
        }
        idt
    };
}


pub fn init() {
	IDT.load();
}

extern "x86-interrupt" fn on_breakpoint(_: InterruptStackFrame) {
	println!("Breakpoint Hit!");
    while crate::sys::keyboard::consume_char().is_none() {sys::timer::pause(0.01)}
} 

extern "x86-interrupt" fn on_double_fault(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    crate::dump_instructions(stack_frame.instruction_pointer, 128);

    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}


extern "x86-interrupt" fn on_page_fault(stack_frame: InterruptStackFrame, ec: PageFaultErrorCode) {
	let ip = stack_frame.instruction_pointer.as_ptr();
    let inst: [u8; 8] = unsafe { core::ptr::read(ip) };
    println!("Code: {:?}", inst);
    panic!("EXCEPTION: PAGE FAULT\n{:#?}\n{:#?}", stack_frame, ec);
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
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    //let mut port = Port::new(0x60);

    let scancode: u8 = inb!(0x60);
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

#[allow(dead_code)]
extern "x86-interrupt" fn default(_: InterruptStackFrame) {
    
}


fn default_irq_handler() {
}


pub fn set_irq_handler(irq: u8, handler: fn()) {
    interrupts::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;

        clear_irq_mask(irq);
    });
}

pub fn set_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });
    unsafe {
        let value = port.read() | (1 << (if irq < 8 { irq } else { irq - 8 }));
        port.write(value);
    }
}

pub fn clear_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });
    unsafe {
        let value = port.read() & !(1 << if irq < 8 { irq } else { irq - 8 });
        port.write(value);
    }
}


#[repr(align(8), C)]
#[derive(Debug, Clone, Default)]
pub struct Registers {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rdi: usize,
    rsi: usize,
    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,
    rbp: usize,
}


macro_rules! wrap {
    ($fn: ident => $w:ident) => {
        #[naked]
        pub unsafe extern "sysv64" fn $w() {
            asm!(
                "push rbp",
                "push rax",
                "push rbx",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "push r12",
                "push r13",
                "push r14",
                "push r15",
                "mov rsi, rsp", // Arg #2: register list
                "mov rdi, rsp", // Arg #1: interupt frame
                "add rdi, 15 * 8",
                "call {}",
                "pop r15",
                "pop r14",
                "pop r13",
                "pop r12",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rbx",
                "pop rax",
                "pop rbp",
                "iretq",
                sym $fn,
                options(noreturn)
            );
        }
    };
}

wrap!(syscall_handler => wrap_syscall);


extern "sysv64" fn syscall_handler(_stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    // The registers order follow the System V ABI convention
    let n    = regs.rax;
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    debug!("Syscall(0x{:08x}, 0x{:08x}, 0x{:08x}, 0x{:08x})", n, arg1, arg2, arg3);

    regs.rax = syscalls::dispatch(n, arg1, arg2, arg3);

    send_eoi(15);
}

