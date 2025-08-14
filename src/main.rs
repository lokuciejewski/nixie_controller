#![no_std]
#![no_main]

use controller_module::{IRSensor, NixieController};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::{
    bind_interrupts,
    gpio::{AnyPin, Level, Output, Pin, Speed},
    i2c::I2c,
    peripherals::I2C1,
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Ticker, Timer};
use panic_probe as _;
mod board_config;

const N_OF_DIGITS: usize = 4;
const SECONDS_TO_SHOW_TIME: u64 = 3;
static MOVEMENT_DETECTED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[cfg(feature = "sample_1")]
use embassy_stm32::peripherals::USART1;
#[cfg(feature = "evalboard")]
use embassy_stm32::peripherals::USART4;

#[cfg(feature = "evalboard")]
bind_interrupts!(struct Irqs {
    I2C1 => embassy_stm32::i2c::EventInterruptHandler<I2C1>, embassy_stm32::i2c::ErrorInterruptHandler<I2C1>;
    USART4 => embassy_stm32::usart::InterruptHandler<USART4>;
});

#[cfg(feature = "sample_1")]
bind_interrupts!(struct Irqs {
    I2C1 => embassy_stm32::i2c::EventInterruptHandler<I2C1>, embassy_stm32::i2c::ErrorInterruptHandler<I2C1>;
    USART1 => embassy_stm32::usart::InterruptHandler<USART1>;
});

#[embassy_executor::task]
pub async fn ir_sensor_task(detection_pin: AnyPin) -> ! {
    let mut ir_sensor = IRSensor::new(detection_pin);
    let mut detection_ticker = Ticker::every(Duration::from_millis(200));
    loop {
        if ir_sensor.movement_detected() {
            info!("Movement detected");
            MOVEMENT_DETECTED.signal(true);
            Timer::after_secs(SECONDS_TO_SHOW_TIME).await;
        } else {
            // MOVEMENT_DETECTED.signal(false);
            detection_ticker.next().await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let mut led_1 = Output::new(led_pin_1!(p), Level::High, embassy_stm32::gpio::Speed::Low);
    let mut _led_2 = Output::new(led_pin_2!(p), Level::High, embassy_stm32::gpio::Speed::Low);

    let mut _button_1 = Input::new(button_pin_1!(p), Pull::Up);
    let mut _button_2 = Input::new(button_pin_2!(p), Pull::Up);
    let mut _button_3 = Input::new(button_pin_3!(p), Pull::Up);

    let reset_pins = [
        Output::new(adapter_reset_pin_0!(p), Level::Low, Speed::High),
        Output::new(adapter_reset_pin_1!(p), Level::Low, Speed::High),
        Output::new(adapter_reset_pin_2!(p), Level::Low, Speed::High),
        Output::new(adapter_reset_pin_3!(p), Level::Low, Speed::High),
    ];

    let mut i2c = I2c::new(
        i2c_instance!(p),
        i2c_scl_pin!(p),
        i2c_sda_pin!(p),
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        Hertz(100_000),
        Default::default(),
    );

    let _serial = embassy_stm32::usart::Uart::new(
        uart_instance!(p),
        uart_rx_pin!(p),
        uart_tx_pin!(p),
        Irqs,
        p.DMA1_CH3,
        p.DMA1_CH4,
        Default::default(),
    );

    let mut nixie_controller: NixieController<'_, _, N_OF_DIGITS> =
        NixieController::new(&mut i2c, hv_en_pin!(p).degrade(), reset_pins);

    while nixie_controller.get_max_number() == 0 {
        match nixie_controller.init_modules().await {
            Ok(_) => (),
            Err(e) => error!("{}", e),
        }
        Timer::after_millis(500).await;
        led_1.toggle();
    }
    info!("Max number: {}", nixie_controller.get_max_number());

    spawner.spawn(ir_sensor_task(p.PA8.degrade())).unwrap();

    info!("Loop start");
    let delay_ms = 250;
    let mut seconds_ticker = Ticker::every(Duration::from_secs(1));

    loop {
        if MOVEMENT_DETECTED.wait().await {
            for _ in 0..SECONDS_TO_SHOW_TIME {
                let time = Instant::now().as_secs() as usize;
                nixie_controller.display_integer(time / 100).await.unwrap();
                info!("Displaying: {:02}", time / 100);
                Timer::after_millis(delay_ms).await;
                nixie_controller.disable_hv();
                Timer::after_millis(delay_ms / 2).await;
                info!("Displaying: {:02}", time % 100);
                nixie_controller.display_integer(time % 100).await.unwrap();
                Timer::after_millis(delay_ms).await;
                nixie_controller.disable_hv();
                Timer::after_millis(delay_ms).await;
                seconds_ticker.next().await;
            }
        }
    }
}
