use std::{
    fs::{self, DirEntry},
    process::Command,
};

use crate::{models::Id, prelude::*};

pub fn command(arg: &str) -> Command {
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };

    let mut cmd = Command::new(shell);

    if cfg!(target_os = "windows") {
        cmd.args(["/C", arg]);
    } else {
        cmd.args([arg]);
    }

    cmd
}

pub struct FileIter(fs::ReadDir);

impl FileIter {
    pub fn new(iter: fs::ReadDir) -> FileIter {
        FileIter(iter)
    }
}

impl Iterator for FileIter {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.0.next() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => return Some(Err(err.into())),
            };

            let meta = match entry.metadata() {
                Ok(meta) => meta,
                Err(err) => return Some(Err(err.into())),
            };

            if meta.is_file() {
                Some(Ok(entry))
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

pub const SIZE: usize = 8;

const LEN: usize = 54;
const MASK: usize = LEN.next_power_of_two() - 1;
const STEP: usize = 8 * SIZE / 5;

pub fn new_id() -> Id {
    static ALPHABET: [char; LEN] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
        'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];

    let mut id = String::new();

    loop {
        fastrand::seed({
            let mut buf = [0; std::mem::size_of::<u64>()];

            let _ = getrandom::getrandom(&mut buf);

            u64::from_le_bytes(buf)
        });

        let mut bytes = [0_u8; STEP];

        for elt in &mut bytes[..] {
            *elt = fastrand::u8(0..=(u8::MAX));
        }

        for &byte in &bytes {
            let byte = byte as usize & MASK;

            if ALPHABET.len() > byte {
                id.push(ALPHABET[byte]);

                if id.len() == SIZE {
                    return Id::from(id);
                }
            }
        }
    }
}

mod fastrand {
    //! A stripped down version of [fastrand](https://github.com/smol-rs/fastrand).

    use std::{
        cell::Cell,
        ops::{Bound, RangeBounds},
    };

    /// Computes `(a * b) >> 32`.
    #[inline]
    fn mul_high_u32(a: u32, b: u32) -> u32 {
        (((a as u64) * (b as u64)) >> 32) as u32
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Rng(u64);

    impl Clone for Rng {
        /// Clones the generator by creating a new generator with the same seed.
        fn clone(&self) -> Rng {
            Rng(self.0)
        }
    }

    impl Rng {
        /// Generates a random `u32`.
        #[inline]
        fn gen_u32(&mut self) -> u32 {
            // gen_u64
            let value = {
                let this = &mut *self;
                // Constants for WyRand taken from: https://github.com/wangyi-fudan/wyhash/blob/master/wyhash.h#L151
                // Updated for the final v4.2 implementation with improved constants for better entropy output.
                const WY_CONST_0: u64 = 0x2d35_8dcc_aa6c_78a5;
                const WY_CONST_1: u64 = 0x8bb8_4b93_962e_acc9;

                let s = this.0.wrapping_add(WY_CONST_0);
                this.0 = s;
                let t = u128::from(s) * u128::from(s ^ WY_CONST_1);
                (t as u64) ^ (t >> 64) as u64
            };

            value as u32
        }

        #[inline]
        fn gen_mod_u32(&mut self, n: u32) -> u32 {
            // Adapted from: https://lemire.me/blog/2016/06/30/fast-random-shuffling/
            let mut r = self.gen_u32();
            let mut hi = mul_high_u32(r, n);
            let mut lo = r.wrapping_mul(n);
            if lo < n {
                let t = n.wrapping_neg() % n;
                while lo < t {
                    r = self.gen_u32();
                    hi = mul_high_u32(r, n);
                    lo = r.wrapping_mul(n);
                }
            }
            hi
        }

        /// Generates a random `u8` in the given range.
        ///
        ///  Panics if the range is empty.
        #[inline]
        pub fn u8(&mut self, range: impl RangeBounds<u8>) -> u8 {
            let panic_empty_range = || {
                panic!(
                    "empty range: {:?}..{:?}",
                    range.start_bound(),
                    range.end_bound()
                )
            };
            let low = match range.start_bound() {
                Bound::Unbounded => u8::MIN,
                Bound::Included(&x) => x,
                Bound::Excluded(&x) => x.checked_add(1).unwrap_or_else(panic_empty_range),
            };
            let high = match range.end_bound() {
                Bound::Unbounded => u8::MAX,
                Bound::Included(&x) => x,
                Bound::Excluded(&x) => x.checked_sub(1).unwrap_or_else(panic_empty_range),
            };
            if low > high {
                panic_empty_range();
            }
            if low == u8::MIN && high == u8::MAX {
                self.gen_u32() as u8
            } else {
                let len = high.wrapping_sub(low).wrapping_add(1);
                low.wrapping_add(self.gen_mod_u32(len as _) as u8)
            }
        }
    }

    // Chosen by fair roll of the dice.
    const DEFAULT_RNG_SEED: u64 = 0xef6f79ed30ba75a;

    /// Make sure the original RNG is restored even on panic.
    struct RestoreOnDrop<'a> {
        rng: &'a Cell<Rng>,
        current: Rng,
    }

    impl Drop for RestoreOnDrop<'_> {
        fn drop(&mut self) {
            self.rng.set(Rng(self.current.0));
        }
    }

    std::thread_local! {
        static RNG: Cell<Rng> = Cell::new(Rng(random_seed().unwrap_or(DEFAULT_RNG_SEED)));
    }

    fn random_seed() -> Option<u64> {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
            thread,
            time::Instant,
        };

        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        thread::current().id().hash(&mut hasher);
        Some(hasher.finish())
    }

    /// Run an operation with the current thread-local generator.
    #[inline]
    fn with_rng<R>(f: impl FnOnce(&mut Rng) -> R) -> R {
        RNG.with(|rng| {
            let current = rng.replace(Rng(0));

            let mut restore = RestoreOnDrop { rng, current };

            f(&mut restore.current)
        })
    }

    /// Initializes the thread-local generator with the given seed.
    #[inline]
    pub fn seed(seed: u64) {
        with_rng(|r| {
            r.0 = seed;
        });
    }

    /// Generates a random `u8` in the given range.
    ///
    /// Panics if the range is empty.
    #[inline]
    pub fn u8(range: impl RangeBounds<u8>) -> u8 {
        with_rng(|r| r.u8(range))
    }
}
