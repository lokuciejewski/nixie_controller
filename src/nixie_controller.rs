use core::{fmt::Debug, marker::PhantomData};

use defmt::{debug, error, info, Format};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker, Timer};
use embedded_hal_async::i2c::I2c;
use heapless::index_map::FnvIndexMap;

use crate::{
    board_config::{HVResources, ModuleResources},
    I2cBus,
};

type DisplaySignal = Signal<CriticalSectionRawMutex, (u16, bool)>;

// Address which newly programmed modules have
const DEFAULT_MODULE_ADDRESS: u8 = 0x20;
// Address to start assigning new modules to
const FIRST_MODULE_ADDRESS: u8 = 0x40;
// Highest possible module address
const LAST_MODULE_ADDRESS: u8 = 0x46;
// Number of digits
const N_OF_DIGITS: usize = 4;

#[embassy_executor::task]
pub async fn nixie_controller_task(
    module_resources: ModuleResources,
    hv_resources: HVResources,
    i2c_bus: &'static I2cBus,
    display_signal: &'static DisplaySignal,
) -> ! {
    let reset_pins = [
        Output::new(module_resources.reset_0, Level::Low, Speed::High),
        Output::new(module_resources.reset_1, Level::Low, Speed::High),
        Output::new(module_resources.reset_2, Level::Low, Speed::High),
        Output::new(module_resources.reset_3, Level::Low, Speed::High),
    ];
    let hv_enable_pin = Output::new(
        hv_resources.hv_en,
        Level::High,
        embassy_stm32::gpio::Speed::VeryHigh,
    );
    let i2c_dev = I2cDevice::new(i2c_bus);
    let mut nixie_controller: NixieController<'_, _, N_OF_DIGITS> = NixieController::new(
        i2c_dev,
        hv_enable_pin,
        reset_pins,
        RefreshPolicy::EveryXMinutes(1),
    );

    while nixie_controller.get_max_number() == 0 {
        match nixie_controller.init_modules().await {
            Ok(_) => (),
            Err(e) => error!("{}", e),
        }
        Timer::after_millis(500).await;
    }
    info!("Max number: {}", nixie_controller.get_max_number());
    match nixie_controller.set_brightness(60).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to set brightness: {:?}", e);
        }
    }
    let mut last_refresh = 0;
    loop {
        // The signal should be set every second
        let (num, comma) = display_signal.wait().await;
        match nixie_controller.refresh_policy {
            RefreshPolicy::EveryXMinutes(x) => {
                if num - last_refresh >= x.into() {
                    match nixie_controller.refresh_all(5).await {
                        Ok(_) => {
                            last_refresh = num;
                        }
                        Err(e) => error!("Module refresh failed : {:?}", e),
                    }
                }
            }
            RefreshPolicy::EveryXHours(x) => {
                if (num / 100) - last_refresh >= x.into() {
                    match nixie_controller.refresh_all(5).await {
                        Ok(_) => {
                            last_refresh = num;
                        }
                        Err(e) => error!("Module refresh failed : {:?}", e),
                    }
                }
            }
            RefreshPolicy::At(hour, minute) => {
                if (num / 100) == hour.into() && (num % 100) == minute.into() {
                    match nixie_controller.refresh_all(5).await {
                        Ok(_) => {
                            last_refresh = num;
                        }
                        Err(e) => error!("Module refresh failed : {:?}", e),
                    }
                }
            }
        }
        match nixie_controller.display_integer(num as usize).await {
            Ok(_) => {}
            Err(e) => error!("Cannot display integer {} : {:?}", num, e),
        }
        match nixie_controller.set_comma(1, comma).await {
            Ok(_) => {}
            Err(_) => error!("Cannot set comma on {} to {}", 1, comma),
        }
    }
}

pub enum NixieControllerError<I2C>
where
    I2C: I2c,
{
    CommunicationError(I2C::Error),
    InvalidParameter,
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
            Self::InvalidParameter => write!(f, "InvalidParameter"),
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
            Self::InvalidParameter => defmt::write!(fmt, "InvalidParameter"),
            Self::InvalidPosition => defmt::write!(fmt, "InvalidPosition"),
            Self::TooManyDigits => defmt::write!(fmt, "TooManyDigits"),
            Self::InternalError => defmt::write!(fmt, "InternalError"),
            Self::ModuleNotFound => defmt::write!(fmt, "ModuleNotFound"),
        }
    }
}

#[allow(unused)]
pub enum RefreshPolicy {
    EveryXMinutes(u8),
    EveryXHours(u8),
    /// Hour, Minute
    At(u8, u8),
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
    refresh_policy: RefreshPolicy,
    i2c: I2C,
}

impl<'nc, I2C, const N: usize> NixieController<'nc, I2C, N>
where
    I2C: I2c,
{
    pub fn new(
        i2c: I2C,
        hv_enable_pin: Output<'nc>,
        mut pin_positions: [Output<'nc>; N],
        refresh_policy: RefreshPolicy,
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
            refresh_policy,
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
        match self.i2c.read(DEFAULT_MODULE_ADDRESS, buf).await {
            Ok(_) => {
                if buf[0] == DEFAULT_MODULE_ADDRESS {
                    debug!(
                        "Device on default address 0x{:02x} validated",
                        DEFAULT_MODULE_ADDRESS
                    );
                    return Ok(DEFAULT_MODULE_ADDRESS);
                } else {
                    debug!(
                            "Device found on default address 0x{:02x} but the register has wrong value: 0x{:02x}",
                            DEFAULT_MODULE_ADDRESS, buf[0]
                        );
                }
            }
            Err(_) => {
                debug!(
                    "Device not found on default address 0x{:02x}",
                    DEFAULT_MODULE_ADDRESS
                );
            }
        }
        for address in FIRST_MODULE_ADDRESS..LAST_MODULE_ADDRESS {
            match self.i2c.read(address, buf).await {
                Ok(_) => {
                    if buf[0] == address {
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
            let next_free_address = self.get_next_free_address();
            module
                .change_address(&mut self.i2c, next_free_address)
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
                        &mut self.i2c,
                        NixieModuleValues::from(
                            (number / (10usize.pow(current_digit as u32)) % 10) as u8,
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

    pub async fn set_comma(&mut self, position: u8, comma_on: bool) -> Result<(), I2C::Error> {
        match self.nixie_modules.get_mut(&(position as u8)) {
            Some(module) => module.set_comma(&mut self.i2c, comma_on).await?,
            None => {}
        }
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

    pub fn get_number_of_modules(&self) -> usize {
        self.nixie_modules.len()
    }

    #[allow(unused)]
    pub async fn get_reported_hv(
        &mut self,
        position_on_display: u8,
    ) -> Result<u16, NixieControllerError<I2C>> {
        let Some(module) = self.nixie_modules.get_mut(&position_on_display) else {
            return Err(NixieControllerError::InvalidPosition);
        };
        module
            .get_hv_reading(&mut self.i2c)
            .await
            .map_err(|e| NixieControllerError::CommunicationError(e))
    }

    pub async fn refresh(
        &mut self,
        position_on_display: u8,
        refresh_delay_ms: u8,
    ) -> Result<(), NixieControllerError<I2C>> {
        self.enable_hv();
        let Some(module) = self.nixie_modules.get_mut(&position_on_display) else {
            return Err(NixieControllerError::InvalidPosition);
        };
        info!(
            "Refreshing module on position: {} with delay 10x{}ms",
            position_on_display, refresh_delay_ms
        );
        let mut ticker = Ticker::every(Duration::from_millis(refresh_delay_ms.into()));
        let current_num = module.displayed_number.clone();
        for i in 0..=9 {
            module
                .display(&mut self.i2c, NixieModuleValues::from(i + 0x80))
                .await
                .map_err(|e| NixieControllerError::CommunicationError(e))?;
            ticker.next().await;
        }
        if current_num != NixieModuleValues::Off {
            module
                .display(&mut self.i2c, current_num)
                .await
                .map_err(|e| NixieControllerError::CommunicationError(e))
        } else {
            self.disable_hv();
            Ok(())
        }
    }

    pub async fn refresh_all(
        &mut self,
        refresh_delay_ms: u8,
    ) -> Result<(), NixieControllerError<I2C>> {
        for i in 0..self.get_number_of_modules() {
            self.refresh(i as u8, refresh_delay_ms).await?;
        }
        Ok(())
    }

    pub async fn set_brightness(
        &mut self,
        brightness_percentage: u8,
    ) -> Result<(), NixieControllerError<I2C>> {
        if brightness_percentage > 100 || brightness_percentage % 5 != 0 {
            return Err(NixieControllerError::InvalidParameter);
        }
        info!("Setting brightness to {}%", brightness_percentage);
        for (_, module) in &mut self.nixie_modules {
            module
                .set_pwm_duty_cycle(&mut self.i2c, brightness_percentage)
                .await
                .map_err(|e| NixieControllerError::CommunicationError(e))?;
        }
        Ok(())
    }
}

#[allow(unused)]
#[repr(u8)]
pub enum NixieModuleRegisters {
    Address = 0x0,
    Value = 0x1,
    HighVoltageLowByte = 0x2,
    HighVoltageHighByte = 0x3,
    PwmValue = 0x4,
    CommaBrightnessCompensation = 0x5,
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

    pub async fn set_comma(&mut self, i2c: &mut I2C, comma_on: bool) -> Result<(), I2C::Error> {
        match self.displayed_number {
            NixieModuleValues::Zero
            | NixieModuleValues::One
            | NixieModuleValues::Two
            | NixieModuleValues::Three
            | NixieModuleValues::Four
            | NixieModuleValues::Five
            | NixieModuleValues::Six
            | NixieModuleValues::Seven
            | NixieModuleValues::Eight
            | NixieModuleValues::Nine
            | NixieModuleValues::Off => {
                if comma_on {
                    i2c.write(
                        self.address,
                        &[
                            NixieModuleRegisters::Value as u8,
                            0x80u8 | self.displayed_number.clone() as u8,
                        ],
                    )
                    .await?;
                    self.displayed_number =
                        NixieModuleValues::from(0x80u8 | self.displayed_number.clone() as u8);
                }
            }
            _ => {
                if !comma_on {
                    i2c.write(
                        self.address,
                        &[
                            NixieModuleRegisters::Value as u8,
                            0x7Fu8 & self.displayed_number.clone() as u8,
                        ],
                    )
                    .await?;
                    self.displayed_number =
                        NixieModuleValues::from(0x7Fu8 & self.displayed_number.clone() as u8);
                }
            }
        }
        Ok(())
    }

    pub async fn get_hv_reading(&mut self, i2c: &mut I2C) -> Result<u16, I2C::Error> {
        let mut buf = [NixieModuleRegisters::HighVoltageLowByte as u8, 0, 0];
        i2c.read(self.address, &mut buf).await?;
        let hv_value = ((buf[2] as u16) << 8) | buf[1] as u16;
        Ok(hv_value)
    }

    pub async fn set_pwm_duty_cycle(
        &mut self,
        i2c: &mut I2C,
        new_duty_cycle_percent: u8,
    ) -> Result<(), I2C::Error> {
        if new_duty_cycle_percent > 100 {
            error!("Invalid duty cycle percent");
            Ok(())
        } else {
            let converted_duty_cycle: u8 = new_duty_cycle_percent / 5;
            debug!("New brightness: {}", converted_duty_cycle);
            i2c.write(
                self.address,
                &[NixieModuleRegisters::PwmValue as u8, converted_duty_cycle],
            )
            .await
        }
    }

    pub async fn set_comma_brightness_compensation(
        &mut self,
        i2c: &mut I2C,
        enable: bool,
    ) -> Result<(), I2C::Error> {
        let mut buf = [
            NixieModuleRegisters::CommaBrightnessCompensation as u8,
            enable as u8,
        ];
        i2c.write(self.address, &mut buf).await
    }
}
