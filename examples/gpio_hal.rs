#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
use embedded_hal::digital::OutputPin;
use hal_mik32::{gpio::port_0::Pin09, rcc::RCC};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    RCC::init(&RCC::default());

    peripherals
        .pm
        .clk_apb_p_set()
        .modify(|_, w| w.gpio_0().enable());

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().pm().enable());

    let mut led = Pin09::new().into_output();
    let _ = led.set_high();

    idle()
}

fn idle() -> ! {
    loop {
        unsafe { asm!("wfi", options(nomem, nostack)) };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    idle()
}

#[unsafe(export_name = "trap_handler")]
fn trap_handler() {}
