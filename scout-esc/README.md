# Scout Electronic Speed Controller

Sout ESC is a firmware for DC brushless motor controllers.

## Setup

## Usage

Ensure you run the commands below from the `scout-esc` directory.

#### Run

`cargo run`

#### Flash

`cargo flash --chip STM32F031C6Tx`

#### Debug

* `openocd`
* `arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/scout-esc`
* `target remote :3333`
* `load`