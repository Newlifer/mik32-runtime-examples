#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo, ptr};
use mik32_pac::{Epic, Gpio8_2, GpioIrq, Peripherals};
use mik32_runtime::entry;

const IRQ_CHANNEL: u32 = 2;
const IRQ_CHANNEL_MASK: u32 = 1 << IRQ_CHANNEL;
const BUTTON_PIN: u32 = 1 << 10;
const BUTTON_MUX: u32 = 1;
const OUTPUT_PIN: u32 = 1 << 7;

static mut OUTPUT_IS_HIGH: bool = false;

pub unsafe fn enable_interrupt() {
    asm!(
        "
            # Enable external interrupts (mie.MEIE <= 1)
            csrr t0, mie # Read the mie register
            li t2, 0x800 # Set the MEIE field (bit 11)
            or t1, t1, t2
            csrw mie, t1 # Update the mie register

            # Enable global interrupts (mstatus.MIE <= 1)
            csrr t0, mstatus # Read the mstatus register
            ori t0, t0, 0x8 # Set MIE field (bit 3)
            csrw mstatus, t0 # Update the mstatus register
        "
    );
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().epic().enable().pm().enable());

    peripherals
        .pm
        .clk_apb_p_set()
        .modify(|_, w| w.gpio_irq().enable().gpio_0().enable().gpio_2().enable());

    peripherals
        .pad_config
        .pad2_cfg()
        .modify(|_, w| w.port_2_7().func1_gpio());
    peripherals
        .pad_config
        .pad0_cfg()
        .modify(|_, w| w.port_0_10().func1_gpio());

    peripherals
        .gpio8_2
        .direction_out()
        .modify(|r, w| unsafe { w.bits(r.bits() | OUTPUT_PIN) });
    peripherals
        .gpio16_0
        .direction_in()
        .modify(|r, w| unsafe { w.bits(r.bits() | BUTTON_PIN) });

    // IRQ channel 2, mux 1 corresponds to PORT_0_10 (the board button).
    peripherals
        .gpio_irq
        .line_mux()
        .write(|w| unsafe { w.bits(BUTTON_MUX << (IRQ_CHANNEL * 4)) });
    peripherals
        .gpio_irq
        .level_clear()
        .write(|w| unsafe { w.bits(IRQ_CHANNEL_MASK) });
    peripherals
        .gpio_irq
        .edge()
        .write(|w| unsafe { w.bits(IRQ_CHANNEL_MASK) });
    peripherals
        .gpio_irq
        .any_edge_clear()
        .write(|w| unsafe { w.bits(IRQ_CHANNEL_MASK) });
    peripherals
        .gpio_irq
        .clear()
        .write(|w| unsafe { w.bits(IRQ_CHANNEL_MASK) });
    peripherals
        .gpio_irq
        .enable_set()
        .write(|w| unsafe { w.bits(IRQ_CHANNEL_MASK) });

    peripherals
        .epic
        .clear()
        .write(|w| w.gpio().clear_bit_by_one());
    peripherals
        .epic
        .mask_level_set()
        .write(|w| w.gpio().set_bit());

    // unsafe {
    //     riscv::register::mie::set_mext();
    //     riscv::interrupt::enable();
    // }

    unsafe { enable_interrupt() };

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

    if epic.raw_status().read().gpio().bit_is_set() {
        let gpio_irq = unsafe { &*GpioIrq::ptr() };

        if gpio_irq.interrupt().read().bits() & IRQ_CHANNEL_MASK != 0 {
            let output_is_high = unsafe { ptr::read_volatile(&raw const OUTPUT_IS_HIGH) };
            let gpio = unsafe { &*Gpio8_2::ptr() };

            if output_is_high {
                gpio.clear().write(|w| unsafe { w.bits(OUTPUT_PIN) });
            } else {
                gpio.set().write(|w| unsafe { w.bits(OUTPUT_PIN) });
            }

            unsafe {
                ptr::write_volatile(&raw mut OUTPUT_IS_HIGH, !output_is_high);
            }
        }

        gpio_irq
            .clear()
            .write(|w| unsafe { w.bits(IRQ_CHANNEL_MASK) });
        epic.clear().write(|w| w.gpio().clear_bit_by_one());
    }
}
