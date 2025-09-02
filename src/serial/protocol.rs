use defmt::{write, Format};
const MAGIC_NUMBER: u8 = 0x69;

#[derive(Debug)]
pub enum MessageError {
    InvalidHeader,
    InvalidCommand,
    InvalidLength,
}

impl Format for MessageError {
    fn format(&self, fmt: defmt::Formatter) {
        write!(fmt, "{:?}", self)
    }
}

#[repr(u8)]
pub enum DeviceMode {
    Normal,
    ExternalControl,
    FirmwareUpdate,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Command {
    SetTime,     // + NaiveTime
    SetDate,     // + NaiveDate
    SetDateTime, // + NaiveDateTime
    GetTime,
    GetDate,
    GetDateTime,
    SetMode, // + DeviceMode
    GetMode,
    DisplayInteger, // + u32
    SetComma,       // + u32 + bool
    GetComma,       // + u32
}

impl Format for Command {
    fn format(&self, fmt: defmt::Formatter) {
        let num = *self as u8;
        write!(fmt, "{}", num)
    }
}

impl TryFrom<u8> for Command {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Command::SetTime),
            1 => Ok(Command::SetDate),
            2 => Ok(Command::SetDateTime),
            3 => Ok(Command::GetTime),
            4 => Ok(Command::GetDate),
            5 => Ok(Command::GetDateTime),
            6 => Ok(Command::SetMode),
            7 => Ok(Command::GetMode),
            8 => Ok(Command::DisplayInteger),
            9 => Ok(Command::SetComma),
            10 => Ok(Command::GetComma),
            _ => Err(MessageError::InvalidCommand),
        }
    }
}

#[repr(packed)]
#[derive(Debug)]
pub struct Header {
    pub magic_number: u8,
    pub id: u8,
    _reserved: u8,
}

impl Header {
    pub fn new(id: u8) -> Self {
        Self {
            magic_number: MAGIC_NUMBER,
            id: id,
            _reserved: 0,
        }
    }
}

#[repr(packed)]
pub struct Message {
    pub header: Header,
    pub command: Command,
    pub payload: [u8; 12],
}

impl Format for Message {
    fn format(&self, fmt: defmt::Formatter) {
        write!(
            fmt,
            "[{}] {}: {}",
            self.header.id, self.command, self.payload
        )
    }
}

impl Message {
    pub fn from_bytes(bytes: [u8; size_of::<Message>()]) -> Result<Self, MessageError> {
        if bytes[0] != MAGIC_NUMBER {
            Err(MessageError::InvalidHeader)
        } else {
            let header = Header {
                magic_number: bytes[0],
                id: bytes[1],
                _reserved: bytes[2],
            };

            let command = Command::try_from(bytes[3])?;
            let mut payload = [0u8; 12];
            payload.copy_from_slice(bytes[4..].iter().as_slice());
            Ok(Message {
                header,
                command,
                payload,
            })
        }
    }

    pub fn to_bytes(&self) -> [u8; size_of::<Self>()] {
        [
            self.header.magic_number,
            self.header.id,
            self.header._reserved,
            self.command as u8,
            self.payload[0],
            self.payload[1],
            self.payload[2],
            self.payload[3],
            self.payload[4],
            self.payload[5],
            self.payload[6],
            self.payload[7],
            self.payload[8],
            self.payload[9],
            self.payload[10],
            self.payload[11],
        ]
    }
}
