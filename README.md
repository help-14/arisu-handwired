# Arisu Handwired

Keyboard firmware for arisu style keyboard handwired using cheap STM32F401.

The firmware is [keyberon](https://github.com/TeXitoi/keyberon).

![arisu handwire](https://i.imgur.com/03L5ocp.jpg)

## Install the rust toolchain
`
curl https://sh.rustup.rs -sSf | sh
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
cargo install cargo-binutils
`


## Compiling
`
cargo objcopy --bin keyberon-f4 --release -- -O binary keyberon.bin
`


## Flashing using DFU
Press boot + restart to get into dfu mode.

`
dfu-util -d 0483:df11 -a 0 --dfuse-address 0x08000000 -D keyberon.bin
`
