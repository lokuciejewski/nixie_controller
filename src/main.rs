#![no_std]
#![no_main]

mod board_config;
mod buttons;
mod ir_sensor;
mod leds;
mod nixie_controller;
mod rtc;

use chrono::{NaiveDateTime, Timelike};
use embassy_stm32::peripherals::USART1;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use static_cell::StaticCell;

use crate::board_config::*;
use crate::buttons::button_task;
use crate::ir_sensor::ir_sensor_task;
use crate::leds::{led_task, set_system_state, SystemState};
use crate::nixie_controller::nixie_controller_task;
use crate::rtc::rtc_task;
use cortex_m_rt::{exception, ExceptionFrame};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, i2c::I2c, peripherals::I2C1};
use embassy_time::Duration;
use panic_probe as _;

type I2cBus =
    Mutex<NoopRawMutex, I2c<'static, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>>;

bind_interrupts!(struct Irqs {
    I2C1 => embassy_stm32::i2c::EventInterruptHandler<I2C1>, embassy_stm32::i2c::ErrorInterruptHandler<I2C1>;
    USART1 => embassy_stm32::usart::InterruptHandler<USART1>;
});

static TIME_CHANGED: Signal<CriticalSectionRawMutex, NaiveDateTime> = Signal::new();
static DISPLAY_SIGNAL: Signal<CriticalSectionRawMutex, (u16, bool)> = Signal::new();

static MOVEMENT_DETECTED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

static BUTTON_1_PRESSED: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static BUTTON_2_PRESSED: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static BUTTON_3_PRESSED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    spawner
        .spawn(ir_sensor_task(ir_sensor_resources!(p), &MOVEMENT_DETECTED))
        .unwrap();
    spawner
        .spawn(button_task(
            button_resources!(p),
            &BUTTON_1_PRESSED,
            &BUTTON_2_PRESSED,
            &BUTTON_3_PRESSED,
        ))
        .unwrap();
    spawner.spawn(led_task(led_resources!(p))).unwrap();

    set_system_state(leds::SystemState::Init);

    let i = i2c_resources!(p);
    let mut i2c_config = embassy_stm32::i2c::Config::default();
    i2c_config.timeout = Duration::from_millis(50);
    let i2c = I2c::new(
        i.i2c_instance,
        i.scl,
        i.sda,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        i2c_config,
    );
    static I2C_BUS: StaticCell<I2cBus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

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

    spawner
        .spawn(rtc_task(rtc_resources!(p), i2c_bus, &TIME_CHANGED))
        .unwrap();
    spawner
        .spawn(nixie_controller_task(
            module_resources!(p),
            hv_resources!(p),
            i2c_bus,
            &DISPLAY_SIGNAL,
        ))
        .unwrap();

    info!("Loop start");

    set_system_state(SystemState::Normal);
    loop {
        let t = TIME_CHANGED.wait().await;
        let int = (t.hour() as u16) * 100 + (t.minute() as u16);
        DISPLAY_SIGNAL.signal((int, t.second() % 2 == 0));
    }
}

#[exception]
unsafe fn HardFault(_frame: &ExceptionFrame) -> ! {
    error!("HardFault!");
    set_system_state(SystemState::Error);
    crate::todo!("Add red led blinking");
}
