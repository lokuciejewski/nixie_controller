use defmt::info;
use embassy_stm32::gpio::{Input, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};

use crate::board_config::IRSensorResources;

type IRSensorSignal = Signal<CriticalSectionRawMutex, bool>;


#[embassy_executor::task]
pub async fn ir_sensor_task(ir_resources: IRSensorResources, ir_sensor_signal: &'static IRSensorSignal) -> ! {
    let mut ir_sensor = IRSensor::new(Input::new(ir_resources.detection, Pull::Up));
    let mut detection_ticker = Ticker::every(Duration::from_millis(200));
    loop {
        if ir_sensor.movement_detected() {
            info!("Movement detected");
            ir_sensor_signal.signal(true);
        } else {
            ir_sensor_signal.signal(false);
            detection_ticker.next().await;
        }
    }
}

pub struct IRSensor<'i> {
    pin: Input<'i>,
}

impl<'i> IRSensor<'i> {
    pub fn new(input_pin: Input<'i>) -> Self {
        Self { pin: input_pin }
    }

    pub fn movement_detected(&mut self) -> bool {
        self.pin.is_low()
    }
}
