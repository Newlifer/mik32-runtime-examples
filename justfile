set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Show available recipes.
default:
    @just --list

build example="gpio":
    cargo build --release --example '{{example}}'

# Build an example and write it to build/firmware.hex.
hex example="gpio":
    New-Item -ItemType Directory -Force build | Out-Null
    cargo objcopy --release --example '{{example}}' -- -O ihex build/firmware.hex

produce example="gpio": (hex example)
