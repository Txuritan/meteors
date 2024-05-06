use std::{ops::Deref, sync::Arc};

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
};

pub struct Data<T>
where
    T: ?Sized,
{
    pub(crate) data: Arc<T>,
}

impl<T> Deref for Data<T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Extractor for Data<T>
where
    T: ?Sized + 'static,
{
    type Error = DataRejection;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(data) = req.data.get::<Data<T>>() {
            Ok(Data {
                data: data.data.clone(),
            })
        } else {
            Err(DataRejection {})
        }
    }
}

pub struct DataRejection {}

impl IntoResponse for DataRejection {
    fn into_response(self) -> HttpResponse {
        "App data is not configured, to configure use App::data()".into_response()
    }
}
