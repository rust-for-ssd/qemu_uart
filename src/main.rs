#![no_std]
#![no_main]

use core::fmt::Write;

use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock {
    locked: AtomicBool,
}

impl Spinlock {
    pub const fn new() -> Self {
        Spinlock {
            locked: AtomicBool::new(false),
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    pub fn lock(&self) -> SpinlockGuard {
        loop {
            match self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
        SpinlockGuard { lock: self }
    }
}

pub struct SpinlockGuard<'a> {
    lock: &'a Spinlock,
}

impl<'a> Drop for SpinlockGuard<'a> {
    #[unsafe(no_mangle)]
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

unsafe impl Sync for Spinlock {}

static LOCK: Spinlock = Spinlock::new();

extern crate panic_halt;

use riscv_rt::entry;

static INIT_PROGRAM_FLAG: AtomicBool = AtomicBool::new(true);

fn set_flag() {
    INIT_PROGRAM_FLAG.store(true, Ordering::SeqCst);
}

fn check_flag() -> bool {
    INIT_PROGRAM_FLAG.load(Ordering::SeqCst)
}

#[unsafe(export_name = "_mp_hook")]
#[rustfmt::skip]
pub extern "Rust" fn user_mp_hook(hartid: usize) -> bool {
    if hartid == 0 {
        true
    } else {
        loop {
            if check_flag() {
                break;
            }
        }
        false
    }
}

use qemu_uart::{uprint, uprintln};

#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        set_flag();
    }
    let id = hartid;

    for _i in 0..10 {
        uprint!("This is from uprint! {}\n", id);
    }
    loop {}
}
