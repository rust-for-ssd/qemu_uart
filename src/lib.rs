#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::Mutex;

pub static mut UART: Uart = Uart {
    uart: Mutex::new(RefCell::new(QemuUart {
        base: 0x10000000,
        thr: 0x10000000 as *mut u8,
        lsr: 0x10000005 as *mut u8,
        lsr_empty_mask: 0x20,
    })),
};

#[non_exhaustive]
pub enum UARTError {
    NonEmptyLSR,
}

pub struct QemuUart {
    base: usize,
    thr: *mut u8,
    lsr: *mut u8,
    lsr_empty_mask: u8,
}

// The following info is specific to the Qemu virt machine.
// The base address is 0x80000000, the UART address base is 0x10000000
// The UART is UART16550
// https://opensocdebug.readthedocs.io/en/latest/02_spec/07_modules/dem_uart/uartspec.html

pub struct Uart {
    pub uart: Mutex<RefCell<QemuUart>>,
}

impl Uart {
    pub fn new(base: usize, lsr_offset: usize, lsr_empty_mask: u8) -> Self {
        Self {
            uart: Mutex::new(RefCell::new(QemuUart {
                base,
                thr: base as *mut u8,
                lsr: (base + lsr_offset) as *mut u8,
                lsr_empty_mask,
            })),
        }
    }
}

impl QemuUart {
    fn try_write_byte(&self, byte: u8) -> Result<(), UARTError> {
        let is_lsr_empty =
            (unsafe { core::ptr::read_volatile(self.lsr) } & self.lsr_empty_mask) != 0;

        if is_lsr_empty {
            unsafe {
                core::ptr::write_volatile(self.thr, byte);
            }
            Ok(())
        } else {
            Err(UARTError::NonEmptyLSR)
        }
    }
}

impl core::fmt::Write for QemuUart {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for byte in s.bytes() {
            while let Err(_) = self.try_write_byte(byte) {}
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! uprint{
    ($($arg:tt)*) => {{
        {
            let mut uart = unsafe {&qemu_uart::UART};
            critical_section::with(|cs| {
                let _ = write!(uart.uart.borrow(cs).borrow_mut(), $($arg)*);
            });
        }
    }};
}

#[macro_export]
macro_rules! uprintln{
    ($($arg:tt)*) => {{
        {
            let mut uart = unsafe {&qemu_uart::UART};
            critical_section::with(|cs| {
                let _ = writeln!(uart.uart.borrow(cs).borrow_mut(), $($arg)*);
            });
        }
    }};
}
