use defmt::{write, Format};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Duration, Ticker};

use crate::{board_config::LedResources, serial::protocol::ProgramMode, StateWatch};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SystemState {
    None,
    Init,
    Normal(ProgramMode),
    Error,
}

impl Format for SystemState {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            SystemState::None => write!(fmt, "None"),
            SystemState::Init => write!(fmt, "Init"),
            SystemState::Normal(mode) => write!(fmt, "Normal ({})", mode),
            SystemState::Error => write!(fmt, "Error"),
        }
    }
}

#[embassy_executor::task]
pub async fn led_task(led_resources: LedResources, current_state: &'static StateWatch) -> ! {
    let mut led_red = Output::new(led_resources.red, Level::High, Speed::Low);
    let mut led_green = Output::new(led_resources.green, Level::High, Speed::Low);

    let mut ticker_short = Ticker::every(Duration::from_millis(100));
    let mut ticker_long = Ticker::every(Duration::from_millis(500));

    let mut state = SystemState::None;
    let mut state_recv = current_state.receiver().unwrap();

    loop {
        if let Some(new_state) = state_recv.try_changed() {
            state = new_state;
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
            SystemState::Normal(mode) => match mode {
                ProgramMode::Clock => {
                    led_green.toggle();
                    led_red.set_low();
                    ticker_long.next().await;
                }
                ProgramMode::ExternalControl => {
                    led_green.set_high();
                    led_red.set_low();
                    ticker_long.next().await;
                }
                ProgramMode::FirmwareUpdate => {
                    led_green.set_low();
                    led_red.set_high();
                    ticker_long.next().await;
                }
            },
            SystemState::Error => {
                led_green.set_low();
                led_red.toggle();
                ticker_short.next().await;
            }
        }
    }
}
