use embassy_stm32::peripherals::*;
use embedded_resources::resource_group;

#[resource_group]
pub(crate) struct ButtonResources {
    button_1: PB10,
    button_2: PB11,
    button_3: PB12,
}

#[resource_group]
pub(crate) struct LedResources {
    red: PB13,
    green: PB14,
}

#[resource_group]
pub(crate) struct IRSensorResources {
    detection: PB9
}

#[resource_group]
pub(crate) struct ModuleResources {
    reset_0: PB0,
    reset_1: PB1,
    reset_2: PB2,
    reset_3: PB3,
}

#[resource_group]
pub(crate) struct I2CResources {
    i2c_instance: I2C1,
    sda: PB7,
    scl: PB8,
}

#[resource_group]
pub(crate) struct UARTResources {
    uart_instance: USART1,
    tx: PA9,
    rx: PA10,
    tx_dma: DMA1_CH3,
    rx_dma: DMA1_CH4,
}

#[resource_group]
pub(crate) struct HVResources {
    hv_en: PA1,
    hv_mon: PA0,
}

#[resource_group]
pub(crate) struct RTCResources {
    sqw_int: PB15,
    int_channel: EXTI15
}