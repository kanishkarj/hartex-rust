#![no_std]
#![no_main]
#![feature(log_syntax)]

extern crate panic_halt;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::hprintln;

use stm32f4::stm32f407;
use stm32f4::stm32f407::interrupt;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use hartex_rust::util::get_msb;
use hartex_rust::messaging;
use hartex_rust::tasks::*;
use hartex_rust::resources::*;
use hartex_rust::types::*;
use hartex_rust::spawn;

#[entry]
fn main() -> ! {
    let app = create(7, 14).unwrap();
    let peripherals = init_peripherals().unwrap();
    let msg1 = messaging::create(7,7,"hello").unwrap();

    spawn!(thread1, 1, msg1, msg1, {
        hprintln!("{:?}", cortex_m::register::control::read().npriv());
        release(&6);
//        msg1.broadcast();
    });
    spawn!(thread2, 2, msg1, msg1, {
        hprintln!("task 2");
        if let Some(x) = msg1.receive() {
            hprintln!("{:?}", x);
        }
    });
    spawn!(thread3, 3, app, app, {
        hprintln!("task 3");
    });

    init(true, &0, |_| loop {
        cortex_m::asm::wfe();
    });

    release(&6);

    start_kernel(unsafe{&mut peripherals.access().unwrap().borrow_mut()}, 150_000);loop {}
}