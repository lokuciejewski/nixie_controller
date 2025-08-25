#![no_std]
#![no_main]

mod board_config;
mod buttons;
mod ir_sensor;
mod leds;
mod nixie_controller;

use embassy_stm32::peripherals::USART1;

use crate::board_config::*;
use crate::buttons::button_task;
use crate::ir_sensor::IRSensor;
use crate::leds::{led_task, set_system_state, SystemState};
use crate::nixie_controller::NixieController;
use cortex_m_rt::{exception, ExceptionFrame};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    i2c::I2c,
    peripherals::I2C1,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Ticker, Timer};
use panic_probe as _;

const N_OF_DIGITS: usize = 4;
const SECONDS_TO_SHOW_TIME: u64 = 3;

static MOVEMENT_DETECTED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

bind_interrupts!(struct Irqs {
    I2C1 => embassy_stm32::i2c::EventInterruptHandler<I2C1>, embassy_stm32::i2c::ErrorInterruptHandler<I2C1>;
    USART1 => embassy_stm32::usart::InterruptHandler<USART1>;
});

#[embassy_executor::task]
pub async fn ir_sensor_task(ir_resources: IRSensorResources) -> ! {
    let mut ir_sensor = IRSensor::new(Input::new(ir_resources.detection, Pull::Up));
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
    spawner
        .spawn(ir_sensor_task(ir_sensor_resources!(p)))
        .unwrap();
    spawner.spawn(button_task(button_resources!(p))).unwrap();
    spawner.spawn(led_task(led_resources!(p))).unwrap();

    set_system_state(leds::SystemState::Init);

    let m = module_resources!(p);
    let reset_pins = [
        Output::new(m.reset_0, Level::Low, Speed::High),
        Output::new(m.reset_1, Level::Low, Speed::High),
        Output::new(m.reset_2, Level::Low, Speed::High),
        Output::new(m.reset_3, Level::Low, Speed::High),
    ];

    let h = hv_resources!(p);
    let hv_enable_pin = Output::new(h.hv_en, Level::High, embassy_stm32::gpio::Speed::VeryHigh);

    let i = i2c_resources!(p);
    let mut i2c_config = embassy_stm32::i2c::Config::default();
    i2c_config.timeout = Duration::from_millis(50);
    let mut i2c = I2c::new(
        i.i2c_instance,
        i.scl,
        i.sda,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        i2c_config,
    );

    let u = uart_resources!(p);
    let _serial = embassy_stm32::usart::Uart::new(
        u.uart_instance,
        u.rx,
        u.tx,
        Irqs,
        p.DMA1_CH3,
        p.DMA1_CH4,
        Default::default(),
    );

    let mut nixie_controller: NixieController<'_, _, N_OF_DIGITS> =
        NixieController::new(&mut i2c, hv_enable_pin, reset_pins);

    while nixie_controller.get_max_number() == 0 {
        match nixie_controller.init_modules().await {
            Ok(_) => (),
            Err(e) => error!("{}", e),
        }
        Timer::after_millis(500).await;
    }
    info!("Max number: {}", nixie_controller.get_max_number());

    info!("Loop start");

    set_system_state(SystemState::Normal);

    let mut ticker = Ticker::every(Duration::from_millis(10000));

    loop {
        // let time = Instant::now().as_secs() as usize;
        // nixie_controller
        //     .display_integer(time % 10000)
        //     .await
        //     .unwrap();
        // info!("Displaying: {:04}", time % 10000);
        // ticker.next().await;
        for i in 0..9 {
            nixie_controller.display_integer(i * 1111).await.unwrap();
            ticker.next().await;
        }
    }
}

#[exception]
unsafe fn HardFault(_frame: &ExceptionFrame) -> ! {
    error!("HardFault!");
    set_system_state(SystemState::Error);
    crate::todo!("Add red led blinking");
    // loop {}
}
