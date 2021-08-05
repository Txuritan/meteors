//! Interior IO util, use at own risk.

use std::io::{Error, ErrorKind, Read, Result, Write};

macro impl_fn($size:ident, $read:ident, $write:ident) {
    pub fn $read<R: std::io::Read>(reader: &mut R) -> std::io::Result<$size> {
        let mut buff: [u8; std::mem::size_of::<$size>()] = [0; std::mem::size_of::<$size>()];

        reader.read_exact(&mut buff)?;

        Ok($size::from_le_bytes(buff))
    }

    pub fn $write<W: std::io::Write>(writer: &mut W, num: $size) -> std::io::Result<()> {
        let bytes: [u8; std::mem::size_of::<$size>()] = num.to_le_bytes();

        writer.write_all(bytes.as_ref())?;

        Ok(())
    }
}

self::impl_fn!(f32, read_f32, write_f32);
self::impl_fn!(f64, read_f64, write_f64);

self::impl_fn!(i8, read_i8, write_i8);
self::impl_fn!(i16, read_i16, write_i16);
self::impl_fn!(i32, read_i32, write_i32);
self::impl_fn!(i64, read_i64, write_i64);
self::impl_fn!(isize, read_isize, write_isize);

self::impl_fn!(u8, read_u8, write_u8);
self::impl_fn!(u16, read_u16, write_u16);
self::impl_fn!(u32, read_u32, write_u32);
self::impl_fn!(u64, read_u64, write_u64);
self::impl_fn!(usize, read_usize, write_usize);

pub fn read_length<R: Read>(reader: &mut R) -> Result<usize> {
    let mut number: u64 = 0;
    let mut count = 0;

    loop {
        let byte = read_u8(reader)?;

        number |= ((byte & 0x7F) as u64) << (7 * count);

        if byte & 0x80 == 0 {
            return Ok(number as usize);
        }

        count += 1;
    }
}

pub fn write_length<W: Write>(writer: &mut W, mut length: usize) -> Result<()> {
    loop {
        let write = (length & 0x7F) as u8;

        length >>= 7;

        if length == 0 {
            write_u8(writer, write)?;

            return Ok(());
        } else {
            write_u8(writer, write | 0x80)?;
        }
    }
}

pub fn read_string<R: Read>(reader: &mut R) -> Result<String> {
    let length = read_length(reader)?;

    let mut buffer = Vec::with_capacity(length);

    for _ in 0..length {
        buffer.push(read_u8(reader)?);
    }

    String::from_utf8(buffer).map_err(|_| Error::from(ErrorKind::InvalidData))
}

pub fn write_string<W: Write>(writer: &mut W, text: &str) -> Result<()> {
    let bytes = text.as_bytes();

    write_length(writer, bytes.len())?;
    writer.write_all(bytes)?;

    Ok(())
}

pub mod structure {
    use {
        crate::{bytes::*, io},
        std::io::{Read, Result, Write},
    };

    macro impl_fn {
        (read, $typ:ident, $read:ident, $value:expr) => {
            pub fn $read<R: Read>(reader: &mut R) -> Result<$typ> {
                crate::assert_byte!(reader, Value::STRING);

                let _field = io::read_string(reader)?;

                crate::assert_byte!(reader, Container::VALUE);
                crate::assert_byte!(reader, $value);

                let value = io::$read(reader)?;

                Ok(value)
            }
        },
        ([$typ:ident, &$typ_ref:ident], $read:ident, $write:ident, $value:expr) => {
            self::impl_fn!(read, $typ, $read, $value);

            pub fn $write<W: Write>(writer: &mut W, key: &str, value: &$typ_ref) -> Result<()> {
                io::write_u8(writer, Value::STRING)?;

                io::write_string(writer, key)?;

                io::write_u8(writer, Container::VALUE)?;
                io::write_u8(writer, $value)?;

                io::$write(writer, value)?;

                Ok(())
            }
        },
        ($typ:ident, $read:ident, $write:ident, $value:expr) => {
            self::impl_fn!(read, $typ, $read, $value);

            pub fn $write<W: Write>(writer: &mut W, key: &str, value: $typ) -> Result<()> {
                io::write_u8(writer, Value::STRING)?;

                io::write_string(writer, key)?;

                io::write_u8(writer, Container::VALUE)?;
                io::write_u8(writer, $value)?;

                io::$write(writer, value)?;

                Ok(())
            }
        },
    }

    self::impl_fn!(f32, read_f32, write_f32, Value::FLOAT_32);
    self::impl_fn!(f64, read_f64, write_f64, Value::FLOAT_64);

    self::impl_fn!(i8, read_i8, write_i8, Value::SIGNED_8);
    self::impl_fn!(i16, read_i16, write_i16, Value::SIGNED_16);
    self::impl_fn!(i32, read_i32, write_i32, Value::SIGNED_32);
    self::impl_fn!(i64, read_i64, write_i64, Value::SIGNED_64);
    self::impl_fn!(isize, read_isize, write_isize, Value::SIGNED_SIZE);

    self::impl_fn!(u8, read_u8, write_u8, Value::UNSIGNED_8);
    self::impl_fn!(u16, read_u16, write_u16, Value::UNSIGNED_16);
    self::impl_fn!(u32, read_u32, write_u32, Value::UNSIGNED_32);
    self::impl_fn!(u64, read_u64, write_u64, Value::UNSIGNED_64);
    self::impl_fn!(usize, read_usize, write_usize, Value::UNSIGNED_SIZE);

    self::impl_fn!([String, &str], read_string, write_string, Value::STRING);
}
