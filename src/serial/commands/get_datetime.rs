use chrono::{Datelike, NaiveDateTime, Timelike};
use defmt::debug;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Receiver};

use crate::serial::protocol::{Command, Message};

pub(crate) fn get_datetime(
    time_changed: &mut Receiver<'_, CriticalSectionRawMutex, NaiveDateTime, 3>,
    current_dt: &mut NaiveDateTime,
) -> Message {
    if let Some(new_time) = time_changed.try_changed() {
        debug!("Time updated in serial thread");
        *current_dt = new_time;
    }
    Message::ack_with_payload(
        0,
        Command::GetDateTime,
        &[
            current_dt.year() as u8,
            (current_dt.year() >> 8) as u8,
            current_dt.month() as u8,
            current_dt.day() as u8,
            current_dt.hour() as u8,
            current_dt.minute() as u8,
            current_dt.second() as u8,
            0,
            0,
            0,
            0,
            0,
        ],
    )
}
