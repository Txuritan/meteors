use {
    crate::{error::InternalError, extractor::Extractor, Error, HttpRequest},
    std::{ops::Deref, sync::Arc},
};

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
            Err(InternalError::InternalServerError(
                "App data is not configured, to configure use App::data()",
            ))
        }
    }
}
