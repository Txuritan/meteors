use std::io::{Read, Write};

use crate::{bytes::*, io, Aloene, Result};

impl Aloene for bool {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        io::assert_byte(reader, Value::BOOL)?;

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
        io::assert_byte(reader, Value::STRING)?;

        io::read_string(reader)
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        io::write_u8(writer, Value::STRING)?;

        io::write_string(writer, self)?;

        Ok(())
    }
}
