#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
use mik32_pac::{Epic, Gpio16_0, Peripherals, Timer32_0};
use mik32_runtime::entry;

const LED_PIN: u32 = 1 << 9;
const TIMER_TOP: u32 = 32_000_000;
const TIMER_OVERFLOW: u32 = 1;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    peripherals
        .pm
        .clk_apb_p_set()
        .write(|w| w.gpio_0().enable());

    peripherals.pm.clk_apb_m_set().write(|w| {
        w.pad_config()
            .enable()
            .pm()
            .enable()
            .timer32_0()
            .enable()
            .epic()
            .enable()
    });

    peripherals
        .pad_config
        .pad0_cfg()
        .modify(|_, w| w.port_0_9().func1_gpio());

    peripherals
        .gpio16_0
        .direction_out()
        .write(|w| unsafe { w.bits(LED_PIN) });

    // At 32 MHz this generates an overflow once per second.
    peripherals
        .timer32_0
        .top()
        .write(|w| unsafe { w.bits(TIMER_TOP) });
    peripherals
        .timer32_0
        .int_mask()
        .write(|w| w.ovf_int().set_bit());

    peripherals
        .epic
        .mask_edge_clear()
        .write(|w| unsafe { w.bits(0xffff) });
    peripherals
        .epic
        .clear()
        .write(|w| unsafe { w.bits(0xffff) });
    peripherals
        .epic
        .mask_edge_set()
        .write(|w| w.timer32_0().enable());

    unsafe {
        riscv::register::mie::set_mext();
        riscv::interrupt::enable();
    }

    peripherals
        .timer32_0
        .enable()
        .write(|w| w.tim_en().enable());

    loop {
        unsafe { asm!("wfi", options(nomem, nostack)) };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe { asm!("wfi", options(nomem, nostack)) };
    }
}

#[unsafe(export_name = "trap_handler")]
fn trap_handler() {
    let epic = unsafe { &*Epic::ptr() };

    if epic.status().read().timer32_0().bit_is_set() {
        let gpio = unsafe { &*Gpio16_0::ptr() };
        gpio.output()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ LED_PIN) });

        let timer = unsafe { &*Timer32_0::ptr() };
        timer
            .int_clear()
            .write(|w| unsafe { w.bits(TIMER_OVERFLOW) });

        epic.clear().write(|w| w.timer32_0().clear_bit_by_one());
    }
}
