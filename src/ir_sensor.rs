use embassy_stm32::gpio::Input;

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
