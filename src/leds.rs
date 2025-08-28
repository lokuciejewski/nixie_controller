use defmt::{info, write, Format};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};

use crate::board_config::LedResources;

pub enum SystemState {
    None,
    Init,
    Normal,
    Error,
}

impl Format for SystemState {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            SystemState::None => write!(fmt, "None"),
            SystemState::Init => write!(fmt, "Init"),
            SystemState::Normal => write!(fmt, "Normal"),
            SystemState::Error => write!(fmt, "Error"),
        }
    }
}

static CURRENT_STATE: Signal<CriticalSectionRawMutex, SystemState> = Signal::new();

pub fn set_system_state(state: SystemState) {
    info!("System state now set to {:?}", state);
    CURRENT_STATE.signal(state);
}

#[embassy_executor::task]
pub async fn led_task(led_resources: LedResources) -> ! {
    let mut led_red = Output::new(led_resources.red, Level::High, Speed::Low);
    let mut led_green = Output::new(led_resources.green, Level::High, Speed::Low);

    let mut ticker_short = Ticker::every(Duration::from_millis(100));
    let mut ticker_long = Ticker::every(Duration::from_millis(500));
    let mut state = SystemState::None;

    loop {
        if CURRENT_STATE.signaled() {
            state = CURRENT_STATE.try_take().unwrap();
        }
        match state {
            SystemState::None => {
                led_green.set_low();
                led_red.set_low();
                ticker_short.next().await;
            }
            SystemState::Init => {
                led_green.toggle();
                led_red.set_low();
                ticker_short.next().await;
            }
            SystemState::Normal => {
                led_green.toggle();
                led_red.set_low();
                ticker_long.next().await;
            }
            SystemState::Error => {
                led_green.set_low();
                led_red.toggle();
                ticker_short.next().await;
            }
        }
    }
}
