#![no_std]
#![no_main]

mod board_config;
mod buttons;
mod ir_sensor;
mod leds;
mod nixie_controller;
mod rtc;
mod serial;

use chrono::{NaiveDateTime, Timelike};
use embassy_stm32::peripherals::USART1;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_sync::watch::Watch;
use static_cell::StaticCell;

use crate::board_config::*;
use crate::buttons::button_task;
use crate::ir_sensor::ir_sensor_task;
use crate::leds::{led_task, SystemState};
use crate::nixie_controller::nixie_controller_task;
use crate::rtc::rtc_task;
use crate::serial::protocol::ProgramMode;
use crate::serial::serial::serial_task;
use cortex_m_rt::{exception, ExceptionFrame};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, i2c::I2c, peripherals::I2C1};
use embassy_time::{Duration, Ticker};
use panic_probe as _;

use defmt::panic;

type I2cBus =
    Mutex<NoopRawMutex, I2c<'static, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>>;

bind_interrupts!(struct Irqs {
    I2C1 => embassy_stm32::i2c::EventInterruptHandler<I2C1>, embassy_stm32::i2c::ErrorInterruptHandler<I2C1>;
    USART1 => embassy_stm32::usart::InterruptHandler<USART1>;
});

type TimeWatch = Watch<CriticalSectionRawMutex, NaiveDateTime, 3>;
type TimeSignal = Signal<CriticalSectionRawMutex, NaiveDateTime>;
type StateWatch = Watch<CriticalSectionRawMutex, SystemState, 3>;

static TIME_WATCH: TimeWatch = Watch::new();
static TIME_SIGNAL_SER_RTC: TimeSignal = Signal::new();
static DISPLAY_SIGNAL: Signal<CriticalSectionRawMutex, (u16, bool)> = Signal::new();

static CURRENT_STATE: StateWatch = Watch::new();

static MOVEMENT_DETECTED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

static BUTTON_1_PRESSED: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static BUTTON_2_PRESSED: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static BUTTON_3_PRESSED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

pub fn set_system_state(state: SystemState) {
    let sender = CURRENT_STATE.sender();
    sender.send(state);
    info!("System state now set to {:?}", state);
}

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
    spawner
        .spawn(led_task(led_resources!(p), &CURRENT_STATE))
        .unwrap();

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

    spawner
        .spawn(serial_task(
            uart_resources!(p),
            &TIME_WATCH,
            &TIME_SIGNAL_SER_RTC,
        ))
        .unwrap();
    spawner
        .spawn(rtc_task(
            rtc_resources!(p),
            i2c_bus,
            &TIME_WATCH,
            &TIME_SIGNAL_SER_RTC,
        ))
        .unwrap();
    spawner
        .spawn(nixie_controller_task(
            module_resources!(p),
            hv_resources!(p),
            i2c_bus,
            &DISPLAY_SIGNAL,
        ))
        .unwrap();

    info!("Main program start");
    set_system_state(SystemState::Normal(ProgramMode::Clock));

    let mut time_changed = TIME_WATCH.receiver().unwrap();
    let mut state_recv = CURRENT_STATE.receiver().unwrap();

    let mut current_state = state_recv.get().await;
    let mut loop_ticker = Ticker::every(Duration::from_millis(500));

    loop {
        if let Some(new_state) = state_recv.try_changed() {
            current_state = new_state;
        }

        match current_state {
            SystemState::None => panic!("SystemState is set to None"),
            SystemState::Init => panic!("SystemState is set to Init"),
            SystemState::Normal(program_mode) => match program_mode {
                ProgramMode::Clock => {
                    let t = time_changed.changed().await;
                    let int = (t.hour() as u16) * 100 + (t.minute() as u16);
                    DISPLAY_SIGNAL.signal((int, t.second() % 2 == 0));
                }
                ProgramMode::ExternalControl => {}
                ProgramMode::FirmwareUpdate => {}
            },
            SystemState::Error => {}
        }

        loop_ticker.next().await;
    }
}

#[exception]
unsafe fn HardFault(_frame: &ExceptionFrame) -> ! {
    error!("HardFault!");
    set_system_state(SystemState::Error);
    loop {}
}
