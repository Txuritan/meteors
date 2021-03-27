use {
    crate::{
        prelude::*,
        regex::REGEX,
        utils::{Reader, Writer},
    },
    flate2::{read::GzDecoder, write::GzEncoder, Compression},
    std::{
        ffi::OsStr,
        fs::{self, File, OpenOptions},
        io::{self, BufReader, BufWriter, Read, Seek as _, SeekFrom, Write},
        mem,
        path::{Path, PathBuf},
    },
};

pub struct StoryFile {
    pub compressed: bool,
    pub file: File,
    pub path: PathBuf,
}

impl StoryFile {
    pub fn new(path: &Path) -> Result<Self> {
        let compressed = {
            let ext = path.extension().and_then(OsStr::to_str);

            match ext {
                Some("html") => false,
                Some("gz") => true,
                ext => anyhow::bail!(
                    "LOGIC ERROR: reached unreachable file extension match with extension `{:?}`",
                    ext
                ),
            }
        };

        Ok(Self {
            compressed,
            file: OpenOptions::new()
                .append(true)
                .read(true)
                .write(true)
                .open(&path)?,
            path: path.to_path_buf(),
        })
    }

    pub fn strip_trackers(&mut self) -> Result<usize> {
        let buf = {
            let mut reader = self.reader();

            let mut buf = Vec::new();

            let _ = reader.read_to_end(&mut buf)?;

            buf
        };

        let _ = self.file.seek(SeekFrom::Start(0))?;

        let positions = REGEX
            .find_iter(&buf[..])
            .map(|(start, end)| start..end)
            .collect::<Vec<_>>();

        let count = positions.len();

        if count != 0 {
            let mut writer = self.writer();

            for (i, byte) in buf.iter().enumerate() {
                if positions.iter().find(|range| range.contains(&i)).is_none() {
                    let _ = writer.write(&[*byte])?;
                }
            }

            writer.flush()?;
        }

        let _ = self.file.seek(SeekFrom::Start(0))?;

        Ok(count)
    }

    pub fn compress(&mut self) -> Result<()> {
        if !self.compressed {
            let _ = self.file.seek(SeekFrom::Start(0))?;

            let new_path = self.path.with_extension("html.gz");
            let mut new_file = OpenOptions::new()
                .append(true)
                .read(true)
                .write(true)
                .create(true)
                .open(&new_path)?;

            {
                let mut reader = BufReader::new(&mut self.file);
                let mut writer = GzEncoder::new(&mut new_file, Compression::best());

                io::copy(&mut reader, &mut writer)?;

                writer.flush()?;
            }

            let old_path = mem::replace(&mut self.path, new_path);
            let _old_file = mem::replace(&mut self.file, new_file);

            let _ = self.file.seek(SeekFrom::Start(0))?;

            fs::remove_file(&old_path)?;
        }

        Ok(())
    }

    pub fn reader(&mut self) -> Reader<BufReader<&mut File>> {
        let base_reader = BufReader::new(&mut self.file);

        if self.compressed {
            Reader::Encoded(GzDecoder::new(base_reader))
        } else {
            Reader::Raw(base_reader)
        }
    }

    pub fn writer(&mut self) -> Writer<BufWriter<&mut File>> {
        let base_writer = BufWriter::new(&mut self.file);

        if self.compressed {
            Writer::Encoded(GzEncoder::new(base_writer, Compression::best()))
        } else {
            Writer::Raw(base_writer)
        }
    }
}
