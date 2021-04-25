include!(concat!(env!("OUT_DIR"), "/models.proto.rs"));

macro_rules! model_some {
    ( $( $field:ident: $type:ty, )+ ) => {
        $(
            model_some!($field: $type);
        )+
    };
    ($field:ident: $type:ty) => {
        pub fn $field(&self) -> &$type {
            match self.$field.as_ref() {
                Some(field) => field,
                None => unreachable!(),
            }
        }
    };
}

impl Story {
    model_some!(info: story::Info, meta: story::Meta,);
}

impl story::Chapter {
    model_some!(content: Range,);
}

impl story::meta::Rating {
    pub const fn class(self) -> &'static str {
        match self {
            story::meta::Rating::Explicit => "explicit",
            story::meta::Rating::Mature => "mature",
            story::meta::Rating::Teen => "teen",
            story::meta::Rating::General => "general",
            story::meta::Rating::NotRated => "not-rated",
            story::meta::Rating::Unknown => "unknown",
        }
    }

    pub const fn symbol(self) -> &'static str {
        match self {
            story::meta::Rating::Explicit => "e",
            story::meta::Rating::Mature => "m",
            story::meta::Rating::Teen => "t",
            story::meta::Rating::General => "g",
            story::meta::Rating::NotRated => "r",
            story::meta::Rating::Unknown => "u",
        }
    }

    pub const fn to(self) -> i32 {
        match self {
            story::meta::Rating::Explicit => 0,
            story::meta::Rating::Mature => 1,
            story::meta::Rating::Teen => 2,
            story::meta::Rating::General => 3,
            story::meta::Rating::NotRated => 4,
            story::meta::Rating::Unknown => 5,
        }
    }
}

impl Range {
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    pub fn from_std(range: std::ops::Range<usize>) -> crate::prelude::Result<Range> {
        use std::convert::TryInto as _;

        Ok(Range {
            start: range.start.try_into()?,
            end: range.end.try_into()?,
        })
    }

    pub fn to_std(&self) -> crate::prelude::Result<std::ops::Range<usize>> {
        use std::convert::TryInto as _;

        Ok((self.start.try_into()?)..(self.end.try_into()?))
    }
}
