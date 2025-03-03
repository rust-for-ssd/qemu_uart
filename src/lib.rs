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
    pub base: usize,
    pub thr: *mut u8,
    pub lsr: *mut u8,
    pub lsr_empty_mask: u8,
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
        while (unsafe { core::ptr::read_volatile(self.lsr) } & self.lsr_empty_mask) == 0 {}

        Ok(())
    }
}

// Needs a critical section to execute
#[macro_export]
macro_rules! csprintln {
    ($cs:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        let uart = &raw const UART;
        unsafe {
            let _ = (*uart)
                .uart
                .borrow_ref_mut($cs)
                .write_fmt(format_args!("{}\n", format_args!($($arg)*)));
        }
    }};
}
