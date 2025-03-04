#![no_std]
#![no_main]

use core::fmt::Write;

use core::sync::atomic::{AtomicBool, Ordering};
use multi_hart_critical_section as _;

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

use qemu_uart::unsafeprintln;

#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        set_flag();
    }
    for i in 0..5 {
        unsafeprintln!("This is from uprint! HID: {}, i: {} **\n", hartid, i);
        unsafeprintln!("This is from uprintln! HID: {}, i: {}", hartid, i);
    }
    loop {}
}
