use {
    crate::{bytes::*, io, Aloene},
    std::io::{Error, ErrorKind, Read, Result, Write},
};

impl Aloene for bool {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        crate::assert_byte!(reader, Value::BOOL);

        let byte = io::read_u8(reader)?;

        Ok(byte == 0x01)
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        io::write_u8(writer, Value::BOOL)?;

        io::write_u8(writer, if *self { 0x01 } else { 0x00 })?;

        Ok(())
    }
}

impl Aloene for String {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        crate::assert_byte!(reader, Value::STRING);

        let length = io::read_length(reader)?;

        let mut buffer = Vec::with_capacity(length);

        for _ in 0..length {
            buffer.push(io::read_u8(reader)?);
        }

        String::from_utf8(buffer).map_err(|_| Error::from(ErrorKind::InvalidData))
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        io::write_u8(writer, Value::STRING)?;

        let bytes = self.as_bytes();

        io::write_length(writer, bytes.len())?;

        writer.write_all(bytes)?;

        Ok(())
    }
}
