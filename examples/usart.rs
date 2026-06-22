#![no_std]
#![no_main]

use core::{fmt::Write, hint::spin_loop, panic::PanicInfo};
use hal_mik32::{
    gpio::port_1::{Pin08, Pin09},
    rcc::RCC,
    usart::{Config, Serial},
};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;

const MESSAGE_DELAY_SPINS: u32 = 500_000;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    RCC::init(&RCC::default());

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().pm().enable());

    // USART1: PORT_1_9 is TX, PORT_1_8 is RX.
    let tx = Pin09::new().into_serial_port();
    let rx = Pin08::new().into_serial_port();
    let serial = Serial::new(peripherals.usart_1, (tx, rx), Config::default()).unwrap();
    let (mut tx, _rx) = serial.split();

    loop {
        let _ = writeln!(tx, "Hello from MIK32 USART1");
        delay(MESSAGE_DELAY_SPINS);
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
