#![no_std]
#![no_main]

use core::sync::atomic::AtomicBool;

use controller_module::{IRSensor, NixieController};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{AnyPin, Level, Output, Pin, Speed},
    i2c::{ErrorInterruptHandler, EventInterruptHandler, I2c},
    peripherals::I2C1,
    rtc::DateTime,
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Ticker, Timer};
use panic_probe as _;

const N_OF_DIGITS: usize = 4;
const SECONDS_TO_SHOW_TIME: u64 = 3;
static MOVEMENT_DETECTED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

bind_interrupts!(struct Irqs {
    // I2C1 => EventInterruptHandler<I2C1>, ErrorInterruptHandler<I2C1>;
    I2C1_EV => EventInterruptHandler<I2C1>;
    I2C1_ER => ErrorInterruptHandler<I2C1>;
});

#[embassy_executor::task]
pub async fn ir_sensor_task(detection_pin: AnyPin) -> ! {
    let mut ir_sensor = IRSensor::new(detection_pin);
    let mut detection_ticker = Ticker::every(Duration::from_millis(200));
    loop {
        if ir_sensor.movement_detected() {
            info!("Movement detected");
            MOVEMENT_DETECTED.signal(true);
            Timer::after_secs(SECONDS_TO_SHOW_TIME).await;
        } else {
            // MOVEMENT_DETECTED.signal(false);
            detection_ticker.next().await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC13, Level::High, embassy_stm32::gpio::Speed::Low);

    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        // p.DMA1_CH1,
        // p.DMA1_CH2,
        p.DMA1_CH6,
        p.DMA1_CH5,
        Hertz(100_000),
        Default::default(),
    );

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
    spawner.spawn(ir_sensor_task(p.PB2.degrade())).unwrap();

    info!("Loop start");
    let delay_ms = 250;
    let mut seconds_ticker = Ticker::every(Duration::from_secs(1));

    loop {
        if MOVEMENT_DETECTED.wait().await {
            for _ in 0..SECONDS_TO_SHOW_TIME {
                let time = Instant::now().as_secs() as usize;
                nixie_controller.display_integer(time / 100).await.unwrap();
                info!("Displaying: {:02}", time / 100);
                Timer::after_millis(delay_ms).await;
                nixie_controller.disable_hv();
                Timer::after_millis(delay_ms / 2).await;
                info!("Displaying: {:02}", time % 100);
                nixie_controller.display_integer(time % 100).await.unwrap();
                Timer::after_millis(delay_ms).await;
                nixie_controller.disable_hv();
                Timer::after_millis(delay_ms).await;
                seconds_ticker.next().await;
            }
        }
    }
}
