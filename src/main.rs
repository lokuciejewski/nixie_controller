#![no_std]
#![no_main]

use controller_module::NixieController;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Pin, Speed},
    i2c::{ErrorInterruptHandler, EventInterruptHandler, I2c},
    peripherals::I2C1,
    time::Hertz,
};
use embassy_time::Timer;
use panic_probe as _;

const N_OF_DIGITS: usize = 4;

bind_interrupts!(struct Irqs {
    I2C1_EV => EventInterruptHandler<I2C1>;
    I2C1_ER => ErrorInterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC13, Level::High, embassy_stm32::gpio::Speed::Low);

    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH5,
        Hertz(100_000),
        Default::default(),
    );
    let delay_ms = 200;

    let reset_pins = [
        Output::new(p.PB6, Level::Low, Speed::Low),
        Output::new(p.PB5, Level::Low, Speed::Low),
        Output::new(p.PB4, Level::Low, Speed::Low),
        Output::new(p.PB3, Level::Low, Speed::Low),
    ];

    let mut nixie_controller: NixieController<'_, _, N_OF_DIGITS> =
        NixieController::new(&mut i2c, p.PB7.degrade(), reset_pins);

    nixie_controller.init_modules().await.unwrap();

    if nixie_controller.get_max_number() == 0 {
        warn!("No modules added to Nixie Controller");
        loop {}
    } else {
        info!("Max number: {}", nixie_controller.get_max_number());
    }
    info!("Loop start");
    loop {
        for i in 0..nixie_controller.get_max_number() {
            nixie_controller.display_integer(i).await.unwrap();
            info!("Displaying: {:02}", i);
            led.set_low();
            Timer::after_millis(delay_ms / 2).await;
            led.set_high();
            Timer::after_millis(delay_ms / 2).await;
        }
    }
}
