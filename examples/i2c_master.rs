#![no_std]
#![no_main]

use core::{hint::spin_loop, panic::PanicInfo};
use embedded_hal::i2c::I2c as _;
use hal_mik32::{
    i2c::{Config, I2c},
    rcc::RCC,
};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;

const SLAVE_ADDRESS: u8 = 0x36;
const TRANSFER_DELAY_SPINS: u32 = 1_000_000;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    RCC::init(&RCC::default());

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().pm().enable());
    peripherals
        .pm
        .clk_apb_p_set()
        .modify(|_, w| w.i2c_1().enable());

    // I2C1: PORT_1_12 is SCL, PORT_1_13 is SDA.
    peripherals.pad_config.pad1_cfg().modify(|_, w| {
        w.port_1_12()
            .func2_interface()
            .port_1_13()
            .func2_interface()
    });
    peripherals
        .pad_config
        .pad1_pupd()
        .modify(|_, w| w.port_1_12().pull_up().port_1_13().pull_up());

    // Approx. 100 kHz with a 32 MHz peripheral clock.
    // The current hal-mik32 API does not expose TIMINGR yet.
    peripherals.i2c_1.timingr().write(|w| unsafe {
        w.presc()
            .bits(3)
            .scldel()
            .bits(4)
            .sdadel()
            .bits(2)
            .sclh()
            .bits(39)
            .scll()
            .bits(39)
    });

    let mut i2c = I2c::new(
        peripherals.i2c_1,
        Config::default().as_master().timeout(100_000),
    );

    let mut counter = 0u8;

    loop {
        let payload = [counter];
        let _ = i2c.write(SLAVE_ADDRESS, &payload);
        counter = counter.wrapping_add(1);
        delay(TRANSFER_DELAY_SPINS);
    }
}

#[inline(always)]
fn delay(spins: u32) {
    for _ in 0..spins {
        spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        spin_loop();
    }
}

#[unsafe(export_name = "trap_handler")]
fn trap_handler() {}
