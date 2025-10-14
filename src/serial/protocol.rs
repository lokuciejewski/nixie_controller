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
    SetDateTime, // + NaiveDateTime
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
            0 => Ok(Command::SetDateTime),
            1 => Ok(Command::GetDateTime),
            2 => Ok(Command::SetMode),
            3 => Ok(Command::GetMode),
            4 => Ok(Command::DisplayInteger),
            5 => Ok(Command::SetComma),
            6 => Ok(Command::GetComma),
            _ => Err(MessageError::InvalidCommand),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MessageType {
    Command,
    Ack,
    Nack,
}

impl Format for MessageType {
    fn format(&self, fmt: defmt::Formatter) {
        let num = *self as u8;
        write!(fmt, "{}", num)
    }
}

impl TryFrom<u8> for MessageType {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::Command),
            1 => Ok(MessageType::Ack),
            2 => Ok(MessageType::Nack),
            _ => Err(MessageError::InvalidCommand),
        }
    }
}

#[repr(packed)]
#[derive(Debug)]
pub struct Header {
    pub magic_number: u8,
    pub id: u8,
    pub message_type: MessageType,
}

impl Header {
    pub fn new(id: u8, message_type: MessageType) -> Self {
        Self {
            magic_number: MAGIC_NUMBER,
            id: id,
            message_type,
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
                message_type: MessageType::try_from(bytes[2])?,
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
            self.header.magic_number as u8,
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
