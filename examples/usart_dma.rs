#![no_std]
#![no_main]

use core::{hint::spin_loop, panic::PanicInfo};

use embedded_hal_nb::{nb::Error as NbError, serial::Write};
use hal_mik32::{
    dma::Dma,
    gpio::port_1::{Pin08, Pin09},
    rcc::RCC,
    usart::{Config, Serial},
};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;

const MESSAGE: &[u8] = b"Hello from MIK32 USART1 + DMA\r\n";
const DMA_TIMEOUT: u32 = 1_000_000;
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
    let tx_pin = Pin09::new().into_serial_port();
    let rx_pin = Pin08::new().into_serial_port();
    let serial = Serial::new(peripherals.usart_1, (tx_pin, rx_pin), Config::default()).unwrap();
    let (mut tx, _rx) = serial.split();

    let mut dma_channel = Dma::new(peripherals.dma).split().channel1;

    loop {
        let transfer = tx.write_dma(dma_channel, MESSAGE).unwrap();
        let (returned_tx, returned_channel, _buffer) = transfer.wait_timeout(DMA_TIMEOUT).unwrap();
        tx = returned_tx;
        dma_channel = returned_channel;

        // DMA completion means that the final byte has reached TXDATA. Wait
        // until USART has shifted it out onto the wire before the delay.
        loop {
            match Write::<u8>::flush(&mut tx) {
                Ok(()) => break,
                Err(NbError::WouldBlock) => spin_loop(),
                Err(NbError::Other(_)) => panic!(),
            }
        }

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
