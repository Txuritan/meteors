use {
    flate2::{read::GzEncoder, Compression},
    std::{
        io::{Error, ErrorKind, Read, Result, Write},
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    },
    tar::{Archive as TarReader, Builder as TarWriter, EntryType, Header},
};

pub struct GzTarReader<R>
where
    R: Read,
{
    inner: TarReader<R>,
}

impl<R> GzTarReader<R>
where
    R: Read,
{
    pub fn new(inner: R) -> GzTarReader<R> {
        Self {
            inner: TarReader::new(inner),
        }
    }

    pub fn get_bytes<P>(&mut self, path: P) -> Result<Option<Vec<u8>>>
    where
        P: AsRef<Path>,
    {
        self.inner
            .entries()?
            .find(|entry| {
                entry
                    .as_ref()
                    .map(|entry| {
                        entry
                            .path()
                            .map(|entry_path| entry_path == path.as_ref())
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .map(|entry| {
                entry.and_then(|mut entry| {
                    let mut bytes = Vec::with_capacity(entry.size() as usize);

                    entry.read_to_end(&mut bytes)?;

                    Ok(bytes)
                })
            })
            .transpose()
    }

    pub fn get_string<P>(&mut self, path: P) -> Result<Option<String>>
    where
        P: AsRef<Path>,
    {
        self.get_bytes(path).and_then(|bytes| {
            bytes
                .map(|bytes| {
                    String::from_utf8(bytes).map_err(|err| Error::new(ErrorKind::InvalidData, err))
                })
                .transpose()
        })
    }

    pub fn into_inner(self) -> R {
        self.inner.into_inner()
    }
}

pub struct GzTarWriter<W>
where
    W: Write,
{
    inner: TarWriter<W>,
    compression: Compression,
}

impl<W> GzTarWriter<W>
where
    W: Write,
{
    pub fn new(inner: W, compression: Compression) -> GzTarWriter<W> {
        Self {
            inner: TarWriter::new(inner),
            compression,
        }
    }

    pub fn append_data<P>(&mut self, path: P, data: &[u8]) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let mut header = Header::new_gnu();

        header.set_gid(0);
        header.set_uid(0);
        header.set_mode(744);
        header.set_size((data.len() + 1) as u64);
        header.set_entry_type(EntryType::Regular);
        header.set_mtime(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|dur| dur.as_secs())
                .unwrap_or(0),
        );
        header.set_cksum();

        self.inner
            .append_data(&mut header, path, GzEncoder::new(data, self.compression))?;

        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
        self.inner.finish()
    }

    pub fn into_inner(self) -> Result<W> {
        self.inner.into_inner()
    }

    pub fn get_ref(&self) -> &W {
        self.inner.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut W {
        self.inner.get_mut()
    }
}
