use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use defmt::{debug, error};

use crate::{
    serial::protocol::{Command, Message},
    TimeSignal,
};

pub(crate) fn set_datetime(message: Message, set_time: &TimeSignal) -> Message {
    let new_date_opt = NaiveDate::from_ymd_opt(
        (((message.payload[1] as u16) << 8) | (message.payload[0] as u16)).into(),
        message.payload[2] as u32,
        message.payload[3] as u32,
    );
    let new_time_opt = NaiveTime::from_hms_opt(
        message.payload[4].into(),
        message.payload[5].into(),
        message.payload[6].into(),
    );
    if let Some(new_date) = new_date_opt {
        if let Some(new_time) = new_time_opt {
            let new_dt = NaiveDateTime::new(new_date, new_time);
            set_time.signal(new_dt);
            debug!("New datetime set");
            Message::ack(0, Command::SetDateTime)
        } else {
            error!("Invalid time");
            Message::nack(0, Command::SetDateTime)
        }
    } else {
        error!("Invalid date");
        Message::nack(0, Command::SetDateTime)
    }
}
