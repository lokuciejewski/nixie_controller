use crate::{board_config::RTCResources, I2cBus};
use crate::{TimeSignal, TimeWatch};

use chrono::Datelike;
use chrono::TimeDelta;
use chrono::Timelike;
use defmt::{error, info};
use ds3231::{Config, DS3231};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;

#[embassy_executor::task]
pub async fn rtc_task(
    rtc_resources: RTCResources,
    i2c_bus: &'static I2cBus,
    current_time: &'static TimeWatch,
    time_set_signal: &'static TimeSignal,
) -> ! {
    info!("Starting RTC task");
    let i2c_dev = I2cDevice::new(i2c_bus);
    let mut rtc = DS3231::new(i2c_dev, 0x68);

    let config = Config {
        time_representation: ds3231::TimeRepresentation::TwentyFourHour,
        square_wave_frequency: ds3231::SquareWaveFrequency::Hz1,
        interrupt_control: ds3231::InterruptControl::SquareWave,
        battery_backed_square_wave: false,
        oscillator_enable: ds3231::Oscillator::Enabled,
    };
    rtc.configure(&config).await.unwrap();

    let mut sqw = ExtiInput::new(rtc_resources.sqw_int, rtc_resources.int_channel, Pull::Up);
    let mut current_dt = rtc.datetime().await.unwrap();

    let time_tick = current_time.sender();
    loop {
        if time_set_signal.signaled() {
            let new_dt = time_set_signal.wait().await;
            info!(
                "Trying to set new DT: [{:02}:{:02}:{:02} {:02}/{:02}/{:04}]",
                new_dt.hour(),
                new_dt.minute(),
                new_dt.second(),
                new_dt.day(),
                new_dt.month(),
                new_dt.year()
            );
            match rtc.set_datetime(&new_dt).await {
                Ok(_) => {
                    info!("DateTime updated succesfully");
                    current_dt = new_dt;
                }
                Err(e) => match e {
                    ds3231::DS3231Error::I2c(_) => error!("Error while updating DateTime [i2c]"),
                    ds3231::DS3231Error::DateTime(_) => error!("Error while updating DateTime [dt]"),
                    ds3231::DS3231Error::Alarm(_) => error!("Error while updating DateTime [alarm]"),
                },
            }
        }
        if current_dt.second() == 0 {
            match rtc.datetime().await {
                Ok(dt) => {
                    info!("Time synced");
                    current_dt = dt;
                }
                Err(_) => {
                    error!("RTC error!");
                    // TODO: Attempt to count the time without the data
                    // TODO: Set error state
                }
            }
        }
        sqw.wait_for_falling_edge().await;
        info!(
            "[{:02}:{:02}:{:02} {:02}/{:02}/{:04}]",
            current_dt.hour(),
            current_dt.minute(),
            current_dt.second(),
            current_dt.day(),
            current_dt.month(),
            current_dt.year()
        );
        time_tick.send(current_dt);
        current_dt += TimeDelta::seconds(1);
    }
}
