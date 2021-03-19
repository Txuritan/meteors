use rand::{rngs::StdRng, Rng as _, SeedableRng as _};

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
        let mut bytes = [0u8; STEP];

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
