# Scout Flight Controller

## Setup

The Scout flight controller prototype software runs on a Nucleo F446RE development board.

## Usage

Ensure you run the commands below from the `scout-fc` directory.

#### Run

`cargo run`

#### Flash

`cargo flash --chip STM32F446RETx`

#### Debug

* `openocd`
* `arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/scout-fc`
* `target remote :3333`
* `load`