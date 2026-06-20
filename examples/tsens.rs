#![no_std]
#![no_main]

use core::{fmt::Write, hint::spin_loop, panic::PanicInfo};
use hal_mik32::{
    gpio::port_1::{Pin08, Pin09},
    rcc::RCC,
    tsens::{ClockSource, Config as TsensConfig, TSENS},
    usart::{Config as UsartConfig, Serial},
};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;

const SENSOR_CLOCK_HZ: u32 = 3_000;
const SENSOR_STARTUP_SPINS: u32 = 100_000;
const SAMPLE_DELAY_SPINS: u32 = 5_000_000;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    let rcc = RCC::default();
    RCC::init(&rcc);

    // TSENS is part of the analog register block, so its APB clock must be on.
    peripherals
        .pm
        .clk_apb_p_set()
        .modify(|_, w| w.analog_regs().enable());

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().pm().enable());

    // USART1: PORT_1_9 is TX, PORT_1_8 is RX, 115200 8N1.
    let tx_pin = Pin09::new().into_serial_port();
    let rx_pin = Pin08::new().into_serial_port();
    let serial = Serial::new(
        peripherals.usart_1,
        (tx_pin, rx_pin),
        UsartConfig::default(),
    );
    let (mut tx, _rx) = serial.split();

    let sensor_config = TsensConfig::default()
        .clock_from_source(ClockSource::OSC32K)
        .with_frequency(SENSOR_CLOCK_HZ);

    let mut sensor = match TSENS::new(peripherals.tsens, &rcc.clocks, sensor_config) {
        Ok(sensor) => sensor,
        Err(error) => {
            let _ = writeln!(tx, "TSENS init error: {:?}", error);
            halt()
        }
    };

    sensor.start_continuous();
    delay(SENSOR_STARTUP_SPINS);

    let _ = writeln!(tx, "TSENS continuous mode started");

    loop {
        let temperature = sensor.get_temperature();
        let _ = writeln!(tx, "Temperature: {} C", temperature);
        delay(SAMPLE_DELAY_SPINS);
    }
}

#[inline(always)]
fn delay(spins: u32) {
    for _ in 0..spins {
        spin_loop();
    }
}

fn halt() -> ! {
    loop {
        spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    halt()
}

#[unsafe(export_name = "trap_handler")]
fn trap_handler() {}
