#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;

const OUTPUT_PIN: u32 = 1 << 9;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    peripherals
        .pm
        .clk_apb_p_set()
        .modify(|_, w| w.gpio_0().enable().gpio_1().enable().analog_regs().enable());

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().pm().enable().wu().enable());

    peripherals
        .gpio16_0
        .direction_out()
        .modify(|r, w| unsafe { w.bits(r.bits() | OUTPUT_PIN) });

    peripherals
        .gpio16_0
        .set()
        .write(|w| unsafe { w.bits(OUTPUT_PIN) });

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
