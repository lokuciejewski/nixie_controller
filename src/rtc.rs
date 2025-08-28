use chrono::Datelike;
use chrono::NaiveDateTime;
use chrono::Timelike;
use defmt::{error, info};
use ds3231::{Config, DS3231};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker};

use crate::{board_config::RTCResources, I2cBus};

type RtcSignal = Signal<CriticalSectionRawMutex, NaiveDateTime>;

#[embassy_executor::task]
pub async fn rtc_task(
    _rtc_resources: RTCResources,
    i2c_bus: &'static I2cBus,
    rtc_signal: &'static RtcSignal,
) -> ! {
    info!("Starting RTC task");
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let i2c_dev = I2cDevice::new(i2c_bus);
    let mut rtc = DS3231::new(i2c_dev, 0x68);
    let config = Config {
        time_representation: ds3231::TimeRepresentation::TwentyFourHour,
        square_wave_frequency: ds3231::SquareWaveFrequency::Hz1,
        interrupt_control: ds3231::InterruptControl::Interrupt,
        battery_backed_square_wave: false,
        oscillator_enable: ds3231::Oscillator::Disabled,
    };
    rtc.configure(&config).await.unwrap();
    loop {
        match rtc.datetime().await {
            Ok(dt) => {
                info!(
                    "[{:02}:{:02}:{:02} {:02}/{:02}/{:04}]",
                    dt.hour(),
                    dt.minute(),
                    dt.second(),
                    dt.day(),
                    dt.month(),
                    dt.year()
                );
                rtc_signal.signal(dt);
            }
            Err(_) => error!("RTC error!"),
        }
        ticker.next().await;
    }
}
