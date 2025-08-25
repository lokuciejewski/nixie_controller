use embassy_stm32::gpio::{Input, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};

use crate::board_config::ButtonResources;

static BUTTON_1: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static BUTTON_2: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static BUTTON_3: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::task]
pub async fn button_task(button_resources: ButtonResources) -> ! {
    let mut button_1 = Input::new(button_resources.button_1, Pull::Up);
    let mut button_2 = Input::new(button_resources.button_2, Pull::Up);
    let mut button_3 = Input::new(button_resources.button_3, Pull::Up);

    let mut ticker_short = Ticker::every(Duration::from_millis(100));
    loop {
        ticker_short.next().await;
    }
}
