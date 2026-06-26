#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use mik32_pac::{Epic, Gpio16_0, GpioIrq, Peripherals};
use mik32_runtime::{entry, interrupt};

const BUTTON_PIN: u32 = 1 << 9;
const LED_PIN: u32 = 1 << 10;

const GPIO_IRQ_LINE: u32 = 1 << 1;
const GPIO_IRQ_LINE_MUX_MASK: u32 = 0x0f << 4;
const GPIO_IRQ_PORT0_9_LINE1: u32 = 1 << 4;

const EPIC_GPIO_IRQ: u32 = 1 << 5;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    peripherals
        .pm
        .clk_apb_p_set()
        .modify(|_, w| w.gpio_0().enable().gpio_irq().enable());

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().pm().enable().epic().enable());

    // P0.9 and P0.10 as GPIO.
    peripherals
        .pad_config
        .pad0_cfg()
        .modify(|_, w| w.port_0_9().func1_gpio().port_0_10().func1_gpio());

    // Active-high button: keep P0.9 low when the button is released.
    peripherals
        .pad_config
        .pad0_pupd()
        .modify(|_, w| w.port_0_9().pull_down().port_0_10().pull_none());

    // P0.10 is output, P0.9 remains input.
    peripherals
        .gpio16_0
        .direction_out()
        .modify(|r, w| unsafe { w.bits((r.bits() | LED_PIN) & !BUTTON_PIN) });

    // Start with the LED off.
    peripherals
        .gpio16_0
        .clear()
        .write(|w| unsafe { w.bits(LED_PIN) });

    // Route P0.9 to GPIO interrupt line 1.
    peripherals.gpio_irq.line_mux().modify(|r, w| unsafe {
        w.bits((r.bits() & !GPIO_IRQ_LINE_MUX_MASK) | GPIO_IRQ_PORT0_9_LINE1)
    });

    // Rising edge on GPIO interrupt line 1.
    peripherals
        .gpio_irq
        .enable_clear()
        .write(|w| unsafe { w.bits(GPIO_IRQ_LINE) });
    peripherals
        .gpio_irq
        .clear()
        .write(|w| unsafe { w.bits(GPIO_IRQ_LINE) });
    peripherals
        .gpio_irq
        .level_set()
        .write(|w| unsafe { w.bits(GPIO_IRQ_LINE) });
    peripherals
        .gpio_irq
        .edge()
        .write(|w| unsafe { w.bits(GPIO_IRQ_LINE) });
    peripherals
        .gpio_irq
        .any_edge_clear()
        .write(|w| unsafe { w.bits(GPIO_IRQ_LINE) });
    peripherals
        .gpio_irq
        .enable_set()
        .write(|w| unsafe { w.bits(GPIO_IRQ_LINE) });

    // Let EPIC forward GPIO IRQs to the core as machine external interrupts.
    peripherals
        .epic
        .mask_edge_clear()
        .write(|w| unsafe { w.bits(EPIC_GPIO_IRQ) });
    peripherals
        .epic
        .clear()
        .write(|w| unsafe { w.bits(EPIC_GPIO_IRQ) });
    peripherals
        .epic
        .mask_edge_set()
        .write(|w| unsafe { w.bits(EPIC_GPIO_IRQ) });

    unsafe {
        interrupt::enable();
    }

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
fn trap_handler() {
    let epic = unsafe { &*Epic::ptr() };

    if epic.status().read().bits() & EPIC_GPIO_IRQ == 0 {
        return;
    }

    let gpio_irq = unsafe { &*GpioIrq::ptr() };

    if gpio_irq.interrupt().read().bits() & GPIO_IRQ_LINE != 0 {
        let gpio = unsafe { &*Gpio16_0::ptr() };

        gpio.set().write(|w| unsafe { w.bits(LED_PIN) });
        gpio_irq.clear().write(|w| unsafe { w.bits(GPIO_IRQ_LINE) });
    }

    epic.clear().write(|w| unsafe { w.bits(EPIC_GPIO_IRQ) });
}
