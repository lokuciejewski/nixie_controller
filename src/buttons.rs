use defmt::info;
use embassy_stm32::gpio::{Input, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};

use crate::board_config::ButtonResources;

type ButtonSignal = Signal<CriticalSectionRawMutex, bool>;

#[embassy_executor::task]
pub async fn button_task(
    button_resources: ButtonResources,
    button_1_signal: &'static ButtonSignal,
    button_2_signal: &'static ButtonSignal,
    button_3_signal: &'static ButtonSignal,
) -> ! {
    let buttons = [
        Input::new(button_resources.button_1, Pull::Up),
        Input::new(button_resources.button_2, Pull::Up),
        Input::new(button_resources.button_3, Pull::Up),
    ];
    let mut states = [false, false, false];
    let signals = [button_1_signal, button_2_signal, button_3_signal];

    let mut ticker_short = Ticker::every(Duration::from_millis(100));

    loop {
        for (idx, button) in buttons.iter().enumerate() {
            if button.is_low() && !states[idx] {
                info!("Button {} pressed", idx);
                states[idx] = true;
                signals[idx].signal(true);
            } else if button.is_high() && states[idx] {
                info!("Button {} released", idx);
                signals[idx].signal(false);
                states[idx] = false;
            }
        }
        ticker_short.next().await;
    }
}
