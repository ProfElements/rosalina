use bit_field::BitField;

pub struct Config;

impl Config {
    pub(crate) fn decrypt_txt_buf(txt_buf: &mut [u8]) {
        let mut key: u32 = 0x73B5DBFA;
        for byte in txt_buf.iter_mut() {
            *byte ^= u8::try_from(key & 0xff).unwrap();
            key = (key << 1) | (key >> 31);
        }
    }
}

pub struct Reader<Data: AsRef<[u8]>> {
    input: Data,
    header: Header,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Header {
    items_count: u16,
}

impl Header {
    pub const LENGTH: usize = 6;

    fn from_bytes(data: [u8; Self::LENGTH]) -> Result<Self, ParseError> {
        if !data.starts_with(b"SCv0") {
            return Err(ParseError::BadMagic);
        }

        let bytes = data.get(4..6).ok_or(ParseError::UnexpectedEnd)?;

        Ok(Self {
            items_count: u16::from_be_bytes(bytes.try_into().unwrap()),
        })
    }
}
#[derive(Debug)]
pub enum ParseError {
    UnexpectedEnd,
    BadMagic,
}

impl<Data: AsRef<[u8]>> Reader<Data> {
    /// # Errors
    /// `UnexpectedEnd`: Data provided is unexpectedly not long enoug
    /// `BadMagic`: the header doesn't have the right magic
    pub fn new(input: Data) -> Result<Self, ParseError> {
        if input.as_ref().len() < Header::LENGTH {
            return Err(ParseError::UnexpectedEnd);
        }

        let header_data = input.as_ref()[0..Header::LENGTH].try_into().unwrap();
        let header = Header::from_bytes(header_data)?;

        //Check offset bounds
        let item_offset_size = core::mem::size_of::<u16>() * usize::from(header.items_count);
        let item_offset_start = Header::LENGTH;
        let item_offset_end = item_offset_start
            .checked_add(item_offset_size)
            .ok_or(ParseError::UnexpectedEnd)?;
        if item_offset_end >= input.as_ref().len() {
            return Err(ParseError::UnexpectedEnd);
        }

        Ok(Self { input, header })
    }

    pub const fn header(&self) -> Header {
        self.header
    }

    fn item_offsets(&self) -> Result<impl ExactSizeIterator<Item = usize> + '_, ParseError> {
        let items_count = usize::from(self.header().items_count);
        let items_offset_end = Header::LENGTH
            .checked_add(items_count * core::mem::size_of::<u16>())
            .ok_or(ParseError::UnexpectedEnd)?;
        let bytes = self
            .input
            .as_ref()
            .get(Header::LENGTH..items_offset_end)
            .ok_or(ParseError::UnexpectedEnd)?;

        Ok(bytes
            .chunks_exact(2)
            .map(|data| usize::from(u16::from_be_bytes(data.try_into().unwrap()))))
    }

    pub fn items(&self) -> impl Iterator<Item = (&'_ str, ConfData)> + '_ {
        self.item_offsets().unwrap().map(move |offset| {
            let item_byte = self.input.as_ref()[offset];

            let name_length = item_byte.get_bits(0..=3);
            let name_bytes_start = offset + 1;
            let name_bytes_end = offset + 1 + usize::from(name_length);
            let name_bytes = self.input.as_ref()[name_bytes_start..=name_bytes_end]
                .try_into()
                .unwrap();

            let item_type = item_byte.get_bits(5..=7);

            let item_data_start = name_bytes_end + 1;

            let item_data_end = match item_type {
                1 | 4 => item_data_start + 2,
                2 | 3 | 7 => item_data_start + 1,
                5 => item_data_start + 4,
                6 => item_data_start + 8,
                _ => item_data_start,
            };

            let item_data = self
                .input
                .as_ref()
                .get(item_data_start..item_data_end)
                .unwrap();
            let items_actual_data = match item_type {
                1 => {
                    let len = u16::from_be_bytes(item_data.try_into().unwrap());
                    ConfData::Array(
                        self.input
                            .as_ref()
                            .get(item_data_end..item_data_end + usize::from(len))
                            .unwrap(),
                    )
                }
                2 => ConfData::Array({
                    //   println!("{:X}", item_data[0]);
                    self.input
                        .as_ref()
                        .get(item_data_end..item_data_end + usize::from(item_data[0]))
                        .unwrap()
                }),
                3 => ConfData::U8(*item_data.first().unwrap()),
                4 => ConfData::U16(u16::from_be_bytes(item_data.try_into().unwrap())),
                5 => ConfData::U32(u32::from_be_bytes(item_data.try_into().unwrap())),
                6 => ConfData::U64(u64::from_be_bytes(item_data.try_into().unwrap())),
                7 => ConfData::Bool(*item_data.first().unwrap() != 0),
                _ => panic!(),
            };

            (core::str::from_utf8(name_bytes).unwrap(), items_actual_data)
        })
    }

    pub fn find(&self, name: &str) -> Option<(&'_ str, ConfData)> {
        self.items().find(|item| item.0 == name)
    }
}
#[derive(Debug)]
pub enum ConfData<'a> {
    Array(&'a [u8]),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Bool(bool),
}
