use {
    crate::{http::HttpRequest, Error},
    std::{ops::Deref, sync::Arc},
};

pub trait Extractor: Sized {
    type Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error>;
}

impl Extractor for () {
    type Error = Error;

    fn extract(_req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(())
    }
}

macro_rules! tuple ({ $($param:ident)* } => {
    impl<$( $param ),*> Extractor for ($( $param, )*)
    where
        $( $param: Extractor<Error = Error>, )*
    {
        type Error = Error;

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

pub struct Body {
    value: String,
}

impl Deref for Body {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Extractor for Body {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        let len = req.inner.body_length().unwrap_or(0);
        let reader = req.inner.as_reader();

        let mut buf = String::with_capacity(len);

        reader.read_to_string(&mut buf)?;

        Ok(Body { value: buf })
    }
}

pub struct Data<T>
where
    T: Send + Sync,
{
    pub(crate) data: Arc<T>,
}

impl<T> Deref for Data<T>
where
    T: Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Extractor for Data<T>
where
    T: Send + Sync + 'static,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(data) = req.data.get::<Data<T>>() {
            Ok(Data {
                data: data.data.clone(),
            })
        } else {
            Err(anyhow::anyhow!("App data is not configured"))
        }
    }
}

pub struct Param<const KEY: &'static str> {
    value: String,
}

impl<const KEY: &'static str> Deref for Param<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for Param<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req.parameters.get(KEY).cloned() {
            Ok(Self { value })
        } else {
            Err(anyhow::anyhow!("Request does not contain route parameter"))
        }
    }
}

pub struct OptionalHeader<const KEY: &'static str> {
    value: Option<String>,
}

impl<const KEY: &'static str> Deref for OptionalHeader<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalHeader<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req
                .inner
                .headers()
                .iter()
                .find(|header| header.field.to_string().eq_ignore_ascii_case(KEY))
                .map(|header| header.value.to_string()),
        })
    }
}

pub struct Query<const KEY: &'static str> {
    value: String,
}

impl<const KEY: &'static str> Deref for Query<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for Query<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req.query.get(KEY).cloned() {
            Ok(Self { value })
        } else {
            Err(anyhow::anyhow!("Request does not contain query value"))
        }
    }
}

pub struct OptionalQuery<const KEY: &'static str> {
    value: Option<String>,
}

impl<const KEY: &'static str> Deref for OptionalQuery<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalQuery<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req.query.get(KEY).cloned(),
        })
    }
}

pub struct RawQuery {
    value: String,
}

impl Deref for RawQuery {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Extractor for RawQuery {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req.raw_query.clone(),
        })
    }
}
