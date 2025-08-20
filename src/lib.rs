#![no_std]
#![no_main]

use core::{fmt::Debug, marker::PhantomData};

use defmt::{debug, error, info, Format};
use embassy_stm32::gpio::{Input, Output};
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use heapless::FnvIndexMap;

// Address which newly programmed modules have
const DEFAULT_MODULE_ADDRESS: u8 = 0x20;
// Address to start assigning new modules to
const FIRST_MODULE_ADDRESS: u8 = 0x40;

pub enum NixieControllerError<I2C>
where
    I2C: I2c,
{
    CommunicationError(I2C::Error),
    InvalidAddress,
    InvalidPosition,
    ModuleNotFound,
    TooManyDigits,
    InternalError,
}

impl<I2C> Debug for NixieControllerError<I2C>
where
    I2C: I2c,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CommunicationError(arg0) => {
                f.debug_tuple("CommunicationError").field(arg0).finish()
            }
            Self::InvalidAddress => write!(f, "InvalidAddress"),
            Self::InvalidPosition => write!(f, "InvalidPosition"),
            Self::TooManyDigits => write!(f, "TooManyDigits"),
            Self::InternalError => write!(f, "InternalError"),
            Self::ModuleNotFound => write!(f, "ModuleNotFound"),
        }
    }
}

impl<I2C> Format for NixieControllerError<I2C>
where
    I2C: I2c,
{
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Self::CommunicationError(_) => {
                defmt::write!(fmt, "CommunicationError")
            }
            Self::InvalidAddress => defmt::write!(fmt, "InvalidAddress"),
            Self::InvalidPosition => defmt::write!(fmt, "InvalidPosition"),
            Self::TooManyDigits => defmt::write!(fmt, "TooManyDigits"),
            Self::InternalError => defmt::write!(fmt, "InternalError"),
            Self::ModuleNotFound => defmt::write!(fmt, "ModuleNotFound"),
        }
    }
}

pub struct NixieController<'nc, I2C, const N: usize>
where
    I2C: I2c,
{
    pin_positions: [Output<'nc>; N],
    nixie_modules: FnvIndexMap<u8, NixieModule<I2C>, N>,
    smallest_unavailable_number: usize,
    available_digits: usize,
    hv_enable: Output<'nc>,
    i2c: &'nc mut I2C,
}

impl<'nc, I2C, const N: usize> NixieController<'nc, I2C, N>
where
    I2C: I2c,
{
    pub fn new(
        i2c: &'nc mut I2C,
        hv_enable_pin: Output<'nc>,
        mut pin_positions: [Output<'nc>; N],
    ) -> Self {
        for pin in pin_positions.iter_mut() {
            pin.set_low(); // Disable all modules when creating the controller
        }
        Self {
            pin_positions,
            nixie_modules: FnvIndexMap::new(),
            smallest_unavailable_number: 1,
            available_digits: 0,
            hv_enable: hv_enable_pin,
            i2c,
        }
    }

    pub async fn init_modules(&mut self) -> Result<(), NixieControllerError<I2C>> {
        for i in 0..N {
            info!("Trying to add module on position {}", i);
            match self.register_new_module(i as u8).await {
                Ok(_) => info!("Module on position {} added", i),
                Err(e) => error!("Failed to add module on position {} -> {:?}", i, e),
            }
        }

        // Enable all modules after registering
        for reset_pin in &mut self.pin_positions {
            reset_pin.set_high();
        }
        Timer::after_millis(50).await;
        debug!("Module initialization complete");
        Ok(())
    }

    async fn find_module_address(&mut self) -> Result<u8, NixieControllerError<I2C>> {
        let buf: &mut [u8; _] = &mut [0xff; 2];
        for address in 0x20..0x50 {
            match self.i2c.read(address, buf).await {
                Ok(_) => {
                    if buf[0] == address || buf[0] == DEFAULT_MODULE_ADDRESS {
                        debug!("Device on address 0x{:02x} validated", address);
                        return Ok(address);
                    } else {
                        debug!(
                            "Device found on address 0x{:02x} but the register has wrong value: 0x{:02x}",
                            address, buf[0]
                        );
                    }
                }
                Err(_) => {
                    debug!("Device not found on address 0x{:02x}", address);
                }
            }
        }
        Err(NixieControllerError::ModuleNotFound)
    }

    fn get_next_free_address(&self) -> u8 {
        let mut first_free_address = FIRST_MODULE_ADDRESS;
        for module in self.nixie_modules.values() {
            if module.address == first_free_address {
                first_free_address += 1;
            } else {
                return first_free_address;
            }
        }
        first_free_address
    }

    fn is_address_free(&self, address: u8) -> bool {
        !self.nixie_modules.iter().any(|(_, m)| m.address == address)
    }

    /// Try registering module on a chosen position
    async fn register_new_module(
        &mut self,
        position_on_display: u8,
    ) -> Result<(), NixieControllerError<I2C>> {
        if self.nixie_modules.contains_key(&position_on_display) || position_on_display as usize > N
        {
            return Err(NixieControllerError::InvalidPosition);
        }

        for pin in &mut self.pin_positions {
            pin.set_low(); // Disable all modules
        }

        Timer::after_millis(10).await;

        self.pin_positions[position_on_display as usize].set_high(); // Enable the module that should be assigned

        Timer::after_millis(100).await;

        let Ok(address) = self.find_module_address().await else {
            return Err(NixieControllerError::ModuleNotFound);
        };

        let mut module = match address {
            DEFAULT_MODULE_ADDRESS => NixieModule::new(),
            _ => NixieModule::with_address(address),
        };
        if module.address == DEFAULT_MODULE_ADDRESS || !self.is_address_free(module.address) {
            module
                .change_address(self.i2c, self.get_next_free_address())
                .await
                .map_err(|e| NixieControllerError::CommunicationError(e))?;
            Timer::after_millis(100).await;
            let mut buf = [NixieModuleRegisters::Address as u8, 0, 0];
            self.i2c
                .read(module.address, &mut buf)
                .await
                .map_err(|e| NixieControllerError::CommunicationError(e))?;
            if buf[1] != module.address {
                return Err(NixieControllerError::InternalError);
            }
            debug!(
                "Module address changed from 0x{:02x} to 0x{:02x}",
                address, module.address
            );
        } else {
            debug!("Module assigned to address 0x{:02x}", module.address);
        }

        self.nixie_modules
            .insert(position_on_display, module)
            .map_err(|_| NixieControllerError::InternalError)
            .map(|_| ())?;
        self.smallest_unavailable_number *= 10;
        self.available_digits += 1;
        Ok(())
    }

    pub fn unregister_module(
        &mut self,
        _module_position: u8,
    ) -> Result<(), NixieControllerError<I2C>> {
        todo!()
    }

    pub async fn display_integer(
        &mut self,
        number: usize,
    ) -> Result<(), NixieControllerError<I2C>> {
        if self.smallest_unavailable_number <= number {
            return Err(NixieControllerError::TooManyDigits);
        }
        self.disable_hv();
        let mut current_digit = 0;
        for position in 0..N {
            match self.nixie_modules.get_mut(&(position as u8)) {
                Some(module) => match module
                    .display(
                        self.i2c,
                        NixieModuleValues::from(
                            (number
                                / (10 * (current_digit as usize) + ((current_digit == 0) as usize))
                                % 10) as u8,
                        ),
                    )
                    .await
                {
                    Ok(_) => {
                        current_digit += 1;
                    }
                    Err(e) => return Err(NixieControllerError::CommunicationError(e)),
                },
                None => {}
            }
        }
        self.enable_hv();
        Ok(())
    }

    pub fn disable_hv(&mut self) {
        self.hv_enable.set_high();
    }

    pub fn enable_hv(&mut self) {
        self.hv_enable.set_low();
    }

    pub fn get_max_number(&self) -> usize {
        self.smallest_unavailable_number - 1
    }

    pub fn get_digit_number(&self) -> usize {
        self.available_digits
    }

    pub async fn get_reported_hv(
        &mut self,
        position_on_display: u8,
    ) -> Result<u16, NixieControllerError<I2C>> {
        let Some(module) = self.nixie_modules.get_mut(&position_on_display) else {
            return Err(NixieControllerError::InvalidPosition);
        };
        module
            .get_hv_reading(self.i2c)
            .await
            .map_err(|e| NixieControllerError::CommunicationError(e))
    }
}

#[repr(u8)]
pub enum NixieModuleRegisters {
    Address = 0x0,
    Value = 0x1,
    HighVoltage = 0x2,
}

#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum NixieModuleValues {
    Zero = 0x0,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    ZeroComma = 0x80,
    OneComma,
    TwoComma,
    ThreeComma,
    FourComma,
    FiveComma,
    SixComma,
    SevenComma,
    EightComma,
    NineComma,
    Off = 0xff,
}

impl From<u8> for NixieModuleValues {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Self::Zero,
            0x1 => Self::One,
            0x2 => Self::Two,
            0x3 => Self::Three,
            0x4 => Self::Four,
            0x5 => Self::Five,
            0x6 => Self::Six,
            0x7 => Self::Seven,
            0x8 => Self::Eight,
            0x9 => Self::Nine,
            0x80 => Self::ZeroComma,
            0x81 => Self::OneComma,
            0x82 => Self::TwoComma,
            0x83 => Self::ThreeComma,
            0x84 => Self::FourComma,
            0x85 => Self::FiveComma,
            0x86 => Self::SixComma,
            0x87 => Self::SevenComma,
            0x88 => Self::EightComma,
            0x89 => Self::NineComma,
            _ => Self::Off,
        }
    }
}

struct NixieModule<I2C>
where
    I2C: I2c,
{
    address: u8,
    displayed_number: NixieModuleValues,
    _i2c_type: PhantomData<I2C>,
}

impl<I2C> NixieModule<I2C>
where
    I2C: I2c,
{
    pub fn new() -> Self {
        Self {
            address: DEFAULT_MODULE_ADDRESS,
            displayed_number: NixieModuleValues::Off,
            _i2c_type: PhantomData::default(),
        }
    }

    pub fn with_address(address: u8) -> Self {
        Self {
            address,
            displayed_number: NixieModuleValues::Off,
            _i2c_type: PhantomData::default(),
        }
    }

    pub async fn change_address(
        &mut self,
        i2c: &mut I2C,
        new_address: u8,
    ) -> Result<(), I2C::Error> {
        if new_address == DEFAULT_MODULE_ADDRESS {
            panic!("Cannot set the default address");
        }
        if new_address != self.address {
            i2c.write(
                self.address,
                &[NixieModuleRegisters::Address as u8, new_address],
            )
            .await?;
            self.address = new_address;
        }

        Ok(())
    }

    pub async fn display(
        &mut self,
        i2c: &mut I2C,
        number: NixieModuleValues,
    ) -> Result<(), I2C::Error> {
        if number != self.displayed_number {
            i2c.write(
                self.address,
                &[NixieModuleRegisters::Value as u8, number.clone() as u8],
            )
            .await?;
            self.displayed_number = number;
        }
        Ok(())
    }

    pub async fn get_hv_reading(&mut self, i2c: &mut I2C) -> Result<u16, I2C::Error> {
        let mut buf = [NixieModuleRegisters::HighVoltage as u8, 0, 0];
        i2c.read(self.address, &mut buf).await?;
        let hv_value = ((buf[2] as u16) << 8) | buf[1] as u16;
        Ok(hv_value)
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
