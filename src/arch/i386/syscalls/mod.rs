use self::calls::SLEEP;

pub mod calls;

#[macro_export]
macro_rules! syscall {
    ($n:expr) => (
        $crate::arch::i386::syscalls::syscall0(
            $n as usize));
    ($n:expr, $a1:expr) => (
        $crate::arch::i386::syscalls::syscall1(
            $n as usize, $a1 as usize));
    ($n:expr, $a1:expr, $a2:expr) => (
        $crate::arch::i386::syscalls::syscall2(
            $n as usize, $a1 as usize, $a2 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => (
        $crate::arch::i386::syscalls::syscall3(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize));
}

pub unsafe fn syscall0(n: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        lateout("rax") res
    );
    res
}

pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1,
        lateout("rax") res
    );
    res
}

pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2,
        lateout("rax") res
    );
    res
}

pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
        lateout("rax") res
    );
    res
}


pub fn dispatch(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    match n {
        SLEEP => {crate::sys::timer::pause((arg1 as f64) / 1000.0); 0}
        _ => {usize::MAX}
    }
}