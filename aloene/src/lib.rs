#[macro_export]
macro_rules! assert_byte {
    ($reader:ident, $byte:expr) => {
        let byte = $crate::io::read_u8($reader)?;

        debug_assert_eq!($byte, byte);
    };
}

pub mod bytes;

pub mod io;

mod impl_element;
mod impl_value;

pub use aloene_macros::Aloene;

pub trait Aloene: Sized {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>;

    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

pub mod test_utils {
    use crate::Aloene;

    #[track_caller]
    pub fn se_de<T: Aloene + std::fmt::Debug + PartialEq>(data: T) {
        let original = data;

        let mut bytes = Vec::new();

        original.serialize(&mut bytes).unwrap();

        let got = T::deserialize(&mut std::io::Cursor::new(bytes)).unwrap();

        assert_eq!(original, got);
    }
}
