// =============================================================== LEDS ======================================================//
#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! led_pin_1 {
    ($peripherals:ident) => {
        $peripherals.PA5 // built-in, D13
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! led_pin_1 {
    ($peripherals:ident) => {
        $peripherals.PB13
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! led_pin_2 {
    ($peripherals:ident) => {
        $peripherals.PA6 // D12
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! led_pin_2 {
    ($peripherals:ident) => {
        $peripherals.PB14
    };
}

// =============================================================== BUTTONS ======================================================//
#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! button_pin_1 {
    ($peripherals:ident) => {
        $peripherals.PC13 // build in, B1
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! button_pin_1 {
    ($peripherals:ident) => {
        $peripherals.PB10
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! button_pin_2 {
    ($peripherals:ident) => {
        $peripherals.PB5 // build in, B1
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! button_pin_2 {
    ($peripherals:ident) => {
        $peripherals.PB11
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! button_pin_3 {
    ($peripherals:ident) => {
        $peripherals.PB4 // build in, B1
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! button_pin_3 {
    ($peripherals:ident) => {
        $peripherals.PB12
    };
}

// =============================================================== RES PINS ======================================================//
#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! adapter_reset_pin_0 {
    ($peripherals:ident) => {
        $peripherals.PC2
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! adapter_reset_pin_0 {
    ($peripherals:ident) => {
        $peripherals.PB0
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! adapter_reset_pin_1 {
    ($peripherals:ident) => {
        $peripherals.PC1
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! adapter_reset_pin_1 {
    ($peripherals:ident) => {
        $peripherals.PB1
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! adapter_reset_pin_2 {
    ($peripherals:ident) => {
        $peripherals.PC3
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! adapter_reset_pin_2 {
    ($peripherals:ident) => {
        $peripherals.PB2
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! adapter_reset_pin_3 {
    ($peripherals:ident) => {
        $peripherals.PC0
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! adapter_reset_pin_3 {
    ($peripherals:ident) => {
        $peripherals.PB3
    };
}

// =============================================================== I2C ======================================================//
#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! i2c_instance {
    ($peripherals:ident) => {
        $peripherals.I2C1
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! i2c_instance {
    ($peripherals:ident) => {
        $peripherals.I2C1
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! i2c_sda_pin {
    ($peripherals:ident) => {
        $peripherals.PB9
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! i2c_sda_pin {
    ($peripherals:ident) => {
        $peripherals.PB7
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! i2c_scl_pin {
    ($peripherals:ident) => {
        $peripherals.PB8
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! i2c_scl_pin {
    ($peripherals:ident) => {
        $peripherals.PB8
    };
}

// =============================================================== UART ======================================================//
#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! uart_instance {
    ($peripherals:ident) => {
        $peripherals.USART4
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! uart_instance {
    ($peripherals:ident) => {
        $peripherals.USART1
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! uart_tx_pin {
    ($peripherals:ident) => {
        $peripherals.PC10
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! uart_tx_pin {
    ($peripherals:ident) => {
        $peripherals.PA9
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! uart_rx_pin {
    ($peripherals:ident) => {
        $peripherals.PC11
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! uart_rx_pin {
    ($peripherals:ident) => {
        $peripherals.PA10
    };
}

// =============================================================== HV ======================================================//
#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! hv_en_pin {
    ($peripherals:ident) => {
        $peripherals.PC6
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! hv_en_pin {
    ($peripherals:ident) => {
        $peripherals.PA1
    };
}

#[cfg(feature = "evalboard")]
#[macro_export]
macro_rules! hv_mon_pin {
    ($peripherals:ident) => {
        $peripherals.PC5
    };
}

#[cfg(feature = "sample_1")]
#[macro_export]
macro_rules! hv_mon_pin {
    ($peripherals:ident) => {
        $peripherals.PA0
    };
}
