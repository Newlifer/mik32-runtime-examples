#![no_std]
#![no_main]

use core::{hint::spin_loop, panic::PanicInfo};
use hal_mik32::{
    i2c::{Config, I2c},
    rcc::RCC,
};
use mik32_pac::Peripherals;
use mik32_runtime::entry;
use riscv as _;
use ssd1306::{
    I2CDisplayInterface, Ssd1306,
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, DisplaySize128x64, WriteOnlyDataCommand},
};

const DISPLAY_ADDRESS: u8 = 0x3c;
const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const FRAME_DELAY_SPINS: u32 = 450_000;

type Display<DI> = Ssd1306<DI, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    RCC::init(&RCC::default()).unwrap();

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

    let i2c = I2c::new(
        peripherals.i2c_1,
        Config::default().as_master().timeout(100_000),
    )
    .unwrap();

    let interface = I2CDisplayInterface::new_custom_address(i2c, DISPLAY_ADDRESS);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    let mut progress = 0u32;

    loop {
        draw_frame(&mut display, progress);
        display.flush().unwrap();

        progress = (progress + 4) % 105;
        delay(FRAME_DELAY_SPINS);
    }
}

fn draw_frame<DI>(display: &mut Display<DI>, progress: u32)
where
    DI: WriteOnlyDataCommand,
{
    display.clear_buffer();

    draw_rect(display, 0, 0, WIDTH, HEIGHT);
    draw_text(display, 18, 8, b"MIK32", 2);
    draw_text(display, 18, 34, b"SSD1306 I2C", 1);
    draw_rect(display, 11, 50, 106, 7);
    fill_rect(display, 12, 51, progress, 5);
}

fn set_pixel<DI>(display: &mut Display<DI>, x: u32, y: u32, on: bool)
where
    DI: WriteOnlyDataCommand,
{
    display.set_pixel(x, y, on);
}

fn fill_rect<DI>(display: &mut Display<DI>, x: u32, y: u32, w: u32, h: u32)
where
    DI: WriteOnlyDataCommand,
{
    for py in y..(y + h).min(HEIGHT) {
        for px in x..(x + w).min(WIDTH) {
            set_pixel(display, px, py, true);
        }
    }
}

fn draw_rect<DI>(display: &mut Display<DI>, x: u32, y: u32, w: u32, h: u32)
where
    DI: WriteOnlyDataCommand,
{
    if w == 0 || h == 0 {
        return;
    }

    let x2 = (x + w - 1).min(WIDTH - 1);
    let y2 = (y + h - 1).min(HEIGHT - 1);

    for px in x..=x2 {
        set_pixel(display, px, y, true);
        set_pixel(display, px, y2, true);
    }

    for py in y..=y2 {
        set_pixel(display, x, py, true);
        set_pixel(display, x2, py, true);
    }
}

fn draw_text<DI>(display: &mut Display<DI>, mut x: u32, y: u32, text: &[u8], scale: u32)
where
    DI: WriteOnlyDataCommand,
{
    for &ch in text {
        draw_char(display, x, y, ch, scale);
        x += 6 * scale;
    }
}

fn draw_char<DI>(display: &mut Display<DI>, x: u32, y: u32, ch: u8, scale: u32)
where
    DI: WriteOnlyDataCommand,
{
    if scale == 0 {
        return;
    }

    let glyph = glyph_5x7(ch);

    for (column, bits) in glyph.iter().enumerate() {
        for row in 0..7 {
            if (bits & (1 << row)) != 0 {
                fill_rect(
                    display,
                    x + column as u32 * scale,
                    y + row * scale,
                    scale,
                    scale,
                );
            }
        }
    }
}

fn glyph_5x7(ch: u8) -> [u8; 5] {
    match ch {
        b'0' => [0x3e, 0x51, 0x49, 0x45, 0x3e],
        b'1' => [0x00, 0x42, 0x7f, 0x40, 0x00],
        b'2' => [0x42, 0x61, 0x51, 0x49, 0x46],
        b'3' => [0x21, 0x41, 0x45, 0x4b, 0x31],
        b'4' => [0x18, 0x14, 0x12, 0x7f, 0x10],
        b'5' => [0x27, 0x45, 0x45, 0x45, 0x39],
        b'6' => [0x3c, 0x4a, 0x49, 0x49, 0x30],
        b'7' => [0x01, 0x71, 0x09, 0x05, 0x03],
        b'8' => [0x36, 0x49, 0x49, 0x49, 0x36],
        b'9' => [0x06, 0x49, 0x49, 0x29, 0x1e],
        b'C' => [0x3e, 0x41, 0x41, 0x41, 0x22],
        b'D' => [0x7f, 0x41, 0x41, 0x22, 0x1c],
        b'I' => [0x00, 0x41, 0x7f, 0x41, 0x00],
        b'K' => [0x7f, 0x08, 0x14, 0x22, 0x41],
        b'M' => [0x7f, 0x02, 0x0c, 0x02, 0x7f],
        b'S' => [0x46, 0x49, 0x49, 0x49, 0x31],
        b' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x00, 0x00, 0x5f, 0x00, 0x00],
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
