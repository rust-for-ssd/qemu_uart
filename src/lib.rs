#![no_std]
#![no_main]

static UART_LOCK: Spinlock = Spinlock::new();

pub enum UARTError {
    NonEmptyLSR,
}

// The following info is specific to the Qemu virt machine.
// The base address is 0x80000000, the UART address base is 0x10000000
// The UART is UART16550
// https://opensocdebug.readthedocs.io/en/latest/02_spec/07_modules/dem_uart/uartspec.html

pub struct Uart {
    base: usize,
    thr: *mut u8,
    lsr: *mut u8,
    lsr_empty_mask: u8,
}

impl Uart {
    pub fn new(base: usize, lsr_offset: usize, lsr_empty_mask: u8) -> Self {
        Self {
            base,
            thr: base as *mut u8,
            lsr: (base + lsr_offset) as *mut u8,
            lsr_empty_mask,
        }
    }

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

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        let lock = UART_LOCK.lock();

        for byte in s.bytes() {
            while let Err(_) = self.try_write_byte(byte) {}
        }
        // Unlock lock
        drop(lock);
        Ok(())
    }
}

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
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

unsafe impl Sync for Spinlock {}
