pub mod body;
pub mod data;
pub mod header;
pub mod param;
pub mod query;

pub use self::{
    body::Body,
    data::Data,
    header::{Header, OptionalHeader},
    param::{Param, OptionalParam},
    query::{OptionalQuery, Query, RawQuery},
};

use crate::HttpRequest;

#[derive(Debug)]
pub enum ExtractorError {
    Missing,
}

pub trait Extractor: Sized {
    type Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error>;
}

impl Extractor for () {
    type Error = ExtractorError;

    fn extract(_req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(())
    }
}

macro_rules! tuple ({ $($param:ident)* } => {
    impl<$( $param ),*> Extractor for ($( $param, )*)
    where
        $( $param: Extractor<Error = ExtractorError>, )*
    {
        type Error = ExtractorError;

        fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
            Ok(($( $param::extract(req)?, )*))
        }
    }
});

tuple! { A }
tuple! { A B }
tuple! { A B C }
tuple! { A B C D }
tuple! { A B C D E }
tuple! { A B C D E F }
tuple! { A B C D E F G }
tuple! { A B C D E F G H }
tuple! { A B C D E F G H I }
tuple! { A B C D E F G H I J }
tuple! { A B C D E F G H I J K }
tuple! { A B C D E F G H I J K L }
