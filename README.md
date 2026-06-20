# Примеры для MIK32

Минимальные Rust-прошивки для микроконтроллера MIK32 «Амур». Проект использует
[`mik32-runtime`](https://github.com/Newlifer/mik32-runtime) как Git-зависимость,
`mik32-pac` для доступа к периферии.

## Примеры

- `gpio` — включает светодиод на 0.9 GPIO пин.
- `gpio-hal` — включает тот же светодиод через GPIO API из `hal-mik32`.
- `gpio-interrupt` — включает светодиод на 0.9 GPIO пин по прерыванию от таймера (timer32 0).
- `usart` — периодически отправляет строку через USART1 (`PORT_1_9`, 115200 8N1).
- `tsens` — периодически измеряет температуру встроенным датчиком и выводит
  результат через USART1 (`PORT_1_9`, 115200 8N1).

Исходники находятся в каталоге [`examples`](examples).

## Подготовка

```console
rustup target add riscv32imc-unknown-none-elf
rustup component add llvm-tools-preview
cargo install cargo-binutils just
```

## Сборка

Напрямую через Cargo:

```console
cargo build --release --example gpio
cargo build --release --example gpio-hal
cargo build --release --example gpio-interrupt
cargo build --release --example usart
cargo build --release --example tsens
```

Или через `just`:

```console
just build
just build gpio-interrupt
```

ELF-файлы создаются в каталоге
`target/riscv32imc-unknown-none-elf/release/examples/`.

## Создание Intel HEX

```console
just produce
just produce gpio-interrupt
```

Также можно использовать эквивалентные команды `just hex` и
`just hex gpio-interrupt`. Выбранная прошивка записывается в
`build/firmware.hex`.
