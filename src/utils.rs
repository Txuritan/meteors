use {
    flate2::{read::GzDecoder, write::GzEncoder},
    rand::{rngs::StdRng, Rng as _, SeedableRng as _},
    std::io::{Read, Write},
};

pub enum Reader<IO>
where
    IO: Read,
{
    Encoded(GzDecoder<IO>),
    Raw(IO),
}

impl<IO> Read for Reader<IO>
where
    IO: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Reader::Encoded(io) => io.read(buf),
            Reader::Raw(io) => io.read(buf),
        }
    }
}

pub enum Writer<IO>
where
    IO: Write,
{
    Encoded(GzEncoder<IO>),
    Raw(IO),
}

impl<IO> Write for Writer<IO>
where
    IO: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Writer::Encoded(io) => io.write(buf),
            Writer::Raw(io) => io.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Writer::Encoded(io) => io.flush(),
            Writer::Raw(io) => io.flush(),
        }
    }
}

// TODO: handle percent encoding
pub fn parse_queries(query: &str) -> Vec<(&str, Option<&str>)> {
    query
        .split('&')
        .map(|param| {
            param
                .contains('=')
                .then(|| {
                    param.find('=').map(|index| {
                        let (key, value) = param.split_at(index);

                        (key, Some(&value[1..]))
                    })
                })
                .flatten()
                .unwrap_or((param, None))
        })
        .collect()
}

pub const SIZE: usize = 8;

const LEN: usize = 54;
const MASK: usize = LEN.next_power_of_two() - 1;
const STEP: usize = 8 * SIZE / 5;

pub fn new_id() -> String {
    static ALPHABET: [char; LEN] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
        'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];

    let mut id = String::new();

    loop {
        let mut rng = StdRng::from_entropy();
        let mut bytes = [0_u8; STEP];

        rng.fill(&mut bytes[..]);

        for &byte in &bytes {
            let byte = byte as usize & MASK;

            if ALPHABET.len() > byte {
                id.push(ALPHABET[byte]);

                if id.len() == SIZE {
                    return id;
                }
            }
        }
    }
}
