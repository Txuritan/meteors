use {
    crate::{bytes::*, io, Aloene},
    std::{
        collections::BTreeMap,
        io::{Error, ErrorKind, Read, Result, Write},
        ops::Range,
    },
};

// TODO: maybe write a container before v is serialized
impl<K: Ord + Aloene, V: Aloene> Aloene for BTreeMap<K, V> {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        assert_byte!(reader, Container::MAP);

        let len = io::read_length(reader)?;

        let mut map = BTreeMap::new();

        for _ in 0..len {
            let key = K::deserialize(reader)?;
            let value = V::deserialize(reader)?;

            map.insert(key, value);
        }

        Ok(map)
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        io::write_u8(writer, Container::MAP)?;

        io::write_length(writer, self.len())?;

        for (key, value) in self.iter() {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T: Aloene> Aloene for Vec<T> {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        assert_byte!(reader, Container::ARRAY);

        let len = io::read_length(reader)?;

        let mut data = Vec::with_capacity(len);

        for _ in 0..len {
            data.push(T::deserialize(reader)?);
        }

        Ok(data)
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        io::write_u8(writer, Container::ARRAY)?;

        io::write_length(writer, self.len())?;

        for item in self.iter() {
            item.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T: Aloene> Aloene for Option<T> {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        match io::read_u8(reader)? {
            Container::NONE => Ok(None),
            Container::SOME => Ok(Some(T::deserialize(reader)?)),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        }
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Some(t) => {
                io::write_u8(writer, Container::SOME)?;

                t.serialize(writer)?;
            }
            None => io::write_u8(writer, Container::NONE)?,
        }

        Ok(())
    }
}

impl Aloene for Range<usize> {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        assert_byte!(reader, Container::STRUCT);

        let start = io::structure::read_usize(reader)?;
        let end = io::structure::read_usize(reader)?;

        Ok(Self { start, end })
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        io::write_u8(writer, Container::STRUCT)?;

        io::structure::write_usize(writer, "start", self.start)?;
        io::structure::write_usize(writer, "end", self.end)?;

        Ok(())
    }
}
