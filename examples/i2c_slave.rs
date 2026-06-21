#![no_std]
#![no_main]

use core::{hint::spin_loop, panic::PanicInfo};
use hal_mik32::{
    i2c::{Config, I2c, SlaveDirection},
    rcc::RCC,
};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;

const SLAVE_ADDRESS: u16 = 0x36;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    RCC::init(&RCC::default());

    peripherals
        .pm
        .clk_apb_m_set()
        .modify(|_, w| w.pad_config().enable().pm().enable());

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

    let mut i2c = I2c::new(
        peripherals.i2c_1,
        Config::default()
            .as_slave()
            .primary_address(SLAVE_ADDRESS)
            .underflow_fill(0x00)
            .timeout(100_000),
    )
    .unwrap();

    let mut value = 0u8;
    let mut receive_buffer = [0u8; 16];

    loop {
        let Ok(request) = i2c.wait_address() else {
            continue;
        };

        match request.direction {
            SlaveDirection::Receive => {
                let _ = i2c.slave_ack();
                if let Ok(transfer) = i2c.slave_receive(&mut receive_buffer) {
                    if transfer.count != 0 {
                        value = receive_buffer[0];
                    }
                }
            }
            SlaveDirection::Transmit => {
                let _ = i2c.slave_transmit(&[value]);
            }
        }
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
