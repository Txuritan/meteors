pub mod body;
pub mod data;
pub mod header;
pub mod param;
pub mod query;

use core::convert::Infallible;

use crate::{
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
};

#[doc(inline)]
pub use self::{
    body::Body,
    data::Data,
    header::{Header, OptionalHeader, ParseHeader},
    param::{OptionalParam, Param, ParseParam},
    query::{OptionalQuery, ParseQuery, Query, RawQuery},
};

pub trait Extractor: Sized {
    type Error: IntoResponse;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error>;
}

impl Extractor for () {
    type Error = Infallible;

    fn extract(_req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(())
    }
}

macro_rules! tuple ({ $($param:ident)* } => {
    impl<$( $param ),*> Extractor for ($( $param, )*)
    where
        $( $param: Extractor, )*
    {
        type Error = HttpResponse;

        #[allow(non_snake_case)]
        fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
            $(
                let $param = match $param::extract(req) {
                    Ok(param) => param,
                    Err(err) => return Err(err.into_response()),
                };
            )*

            Ok(($( $param, )*))
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
