// =============================================================== LEDS ======================================================//
#[macro_export]
macro_rules! led_pin_1 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PA5.degrade() // built-in, D13
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB13.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

#[macro_export]
macro_rules! led_pin_2 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PA6.degrade() // D12
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB14.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

// =============================================================== BUTTONS ======================================================//
#[macro_export]
macro_rules! button_pin_1 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PC13.degrade() // build in, B1
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB10.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

#[macro_export]
macro_rules! button_pin_2 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PB5.degrade()
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB11.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

#[macro_export]
macro_rules! button_pin_3 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PB4.degrade()
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB12.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

// =============================================================== RES PINS ======================================================//
#[macro_export]
macro_rules! adapter_reset_pin_0 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PA3.degrade()
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB0.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

#[macro_export]
macro_rules! adapter_reset_pin_1 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PA2.degrade()
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB1.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

#[macro_export]
macro_rules! adapter_reset_pin_2 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PA10.degrade()
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB2.degrade()
        } else {
            crate::panic!("unsupported")
        }
    };
}

#[macro_export]
macro_rules! adapter_reset_pin_3 {
    ($peripherals:ident) => {
        if cfg!(feature = "evalboard") {
            $peripherals.PB3.degrade()
        } else if cfg!(feature = "sample_1") {
            $peripherals.PB3.degrade()
        } else {
            crate::panic!("unsupported")
        }
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




