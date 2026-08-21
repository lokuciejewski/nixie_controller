# Controller/clock for mini Nixie I2C modules

STM32-based controller board that uses 4 [nixie modules](https://github.com/lokuciejewski/nixie_module) to display the current time. The modules can be changed at any time the board is off and swapped if the nixie tube goes bad or the user wants to insert a different nixie tube.

![V1 board with 4 modules](./media/controller.jpg)

The board uses a STM32U031 microcontroller coupled with DS3231 RTC to achieve low power, reliable timekeeping.
For High Voltage for the nixie tubes, it uses a premade nixie PSU 160-180V (available on aliexpress). It uses a single cell CR2032 battery to keep the time when the module is not connected. For communication, it uses a serial interface over the USB-C port (which also serves as the power input). The board features 3 programmable buttons and two LEDs for signalling the system status.

![V1 empty board](./media/controller_empty.jpg)
Note: V1 version of the board contains multiple test points and additional inputs which will be removed in the V2 version.

## Building

Prerequisites:

- Rust
- `cargo`

To build, simply run `cargo build --release`

## Flashing

The board is flashed using `probe-rs`, which can utilize a range of multiple programmers, for example an ST-Link.
Connect the programmer to the J3 and run `cargo run --release`.

## Operation

Currently, the time can only be set using the serial port and the `comm.py` script.

The serial protocol for talking to the board can be found [in the `serial` directory](./src/serial/)

## Planned features

V2 will have an option to wire 1 or 2 neon tubes between the hour and minute displays (since not all nixie tubes come with a comma).

It may also be equipped with the human presence sensor to only light up when it can detect a human.

A console interface is planned to utilize all features of the board (such as programming inputs, time, etc.)

The board has a second serial available on pins which may be used for wireless time access via e.g. an esp32.

![V1 board with nixies off](./media/controller_off.jpg)
