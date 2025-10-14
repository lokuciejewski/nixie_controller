use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use defmt::{debug, error, info};
use embassy_time::{Duration, Ticker};

use crate::{
    board_config::UARTResources,
    serial::protocol::{Command, Header, Message, MessageType},
    Irqs, TimeSignal, TimeWatch,
};

#[embassy_executor::task]
pub async fn serial_task(
    uart_resources: UARTResources,
    current_time: &'static TimeWatch,
    set_time: &'static TimeSignal,
) -> ! {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut serial = embassy_stm32::usart::Uart::new(
        uart_resources.uart_instance,
        uart_resources.rx,
        uart_resources.tx,
        Irqs,
        uart_resources.tx_dma,
        uart_resources.rx_dma,
        Default::default(),
    )
    .unwrap();

    let mut serial_buffer = [0u8; size_of::<Message>()];

    let mut time_changed = current_time.receiver().unwrap();
    let mut current_dt = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    );

    loop {
        match serial.read(&mut serial_buffer).await {
            Ok(_) => match Message::from_bytes(serial_buffer) {
                Ok(message) => {
                    info!("{}", message);
                    match message.command {
                        crate::serial::protocol::Command::SetDateTime => {
                            let new_date_opt = NaiveDate::from_ymd_opt(
                                (((message.payload[1] as u16) << 8) | (message.payload[0] as u16))
                                    .into(),
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
                                } else {
                                    error!("Invalid time");
                                }
                            } else {
                                error!("Invalid date");
                            }
                        }
                        crate::serial::protocol::Command::GetDateTime => {
                            if let Some(new_time) = time_changed.try_changed() {
                                debug!("Time updated in serial thread");
                                current_dt = new_time;
                            }
                            let response = Message {
                                header: Header::new(0, MessageType::Ack),
                                command: Command::GetDateTime,
                                payload: [
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
                            };
                            match serial.write(&response.to_bytes()).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Could not send datetime: {}", e);
                                }
                            }
                        }
                        crate::serial::protocol::Command::SetMode => todo!(),
                        crate::serial::protocol::Command::GetMode => todo!(),
                        crate::serial::protocol::Command::DisplayInteger => todo!(),
                        crate::serial::protocol::Command::SetComma => todo!(),
                        crate::serial::protocol::Command::GetComma => todo!(),
                    }
                }
                Err(e) => {
                    error!("Invalid message: {}", e);
                }
            },
            Err(e) => {
                error!("Serial error: {}", e);
            }
        }
        ticker.next().await;
    }
}
