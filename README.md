# Минимальный проект для MIK32

Проект использует `mik32-runtime` как Git-зависимость и собирается для
`riscv32imc-unknown-none-elf`.

```console
cargo build --release --example gpio
cargo build --release --example gpio-interrupt
```

Для создания Intel HEX (требуется `cargo-binutils`):

```console
cargo hex-gpio
cargo hex-gpio-interrupt
```
