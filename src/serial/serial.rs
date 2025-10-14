use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use defmt::{debug, error};
use embassy_time::{Duration, Ticker};

use crate::{
    board_config::UARTResources,
    serial::{
        commands::{get_datetime::get_datetime, set_datetime::set_datetime},
        protocol::Message,
    },
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

    let mut time_changed: embassy_sync::watch::Receiver<
        '_,
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        NaiveDateTime,
        3,
    > = current_time.receiver().unwrap();
    let mut current_dt = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    );

    loop {
        match serial.read(&mut serial_buffer).await {
            Ok(_) => match Message::from_bytes(serial_buffer) {
                Ok(message) => {
                    debug!("Message received: {}", message);
                    let response = match message.command {
                        crate::serial::protocol::Command::SetDateTime => {
                            set_datetime(message, set_time)
                        }
                        crate::serial::protocol::Command::GetDateTime => {
                            get_datetime(&mut time_changed, &mut current_dt)
                        }
                        crate::serial::protocol::Command::SetMode => todo!(),
                        crate::serial::protocol::Command::GetMode => todo!(),
                        crate::serial::protocol::Command::DisplayInteger => todo!(),
                        crate::serial::protocol::Command::SetComma => todo!(),
                        crate::serial::protocol::Command::GetComma => todo!(),
                    };
                    match serial.write(&response.to_bytes()).await {
                        Ok(_) => debug!("Response sent"),
                        Err(e) => error!("Failed to send response: {}", e),
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
