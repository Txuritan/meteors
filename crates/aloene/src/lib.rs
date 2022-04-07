#![feature(decl_macro)]

pub mod bytes;
pub mod io;

mod error;

mod impl_element;
mod impl_value;

pub use aloene_macros::Aloene;

pub use crate::error::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub trait Aloene: Sized {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> Result<Self>;

    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()>;
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
