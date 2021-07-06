use {
    crate::{http::StatusCode, HttpResponse},
    std::{error, fmt},
};

pub struct Error {
    inner: Box<dyn ResponseError>,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl error::Error for Error {}

impl<T> From<T> for Error
where
    T: ResponseError + 'static,
{
    fn from(err: T) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}

pub trait ResponseError: fmt::Debug + fmt::Display {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status_code())
            .header("Content-Type", "text/plain; charset=utf-8")
            .body({
                use std::io::Write as _;

                let mut buf = Vec::new();

                let _ = write!(&mut buf, "{}", self);

                buf
            })
    }
}

impl ResponseError for Box<dyn error::Error + 'static> {}

macro_rules! internal_error {
    ($( $name:ident[$status:expr], )*) => {
        $(
            #[allow(non_snake_case)]
            pub fn $name(err: T) -> Error {
                InternalError { cause: err, status: $status }.into()
            }
        )*
    };
}

pub struct InternalError<T> {
    cause: T,
    status: StatusCode,
}

impl<T> InternalError<T>
where
    T: fmt::Debug + fmt::Display + 'static,
{
    internal_error![
        BadRequest[StatusCode::BAD_REQUEST],
        Unauthorized[StatusCode::UNAUTHORIZED],
        PaymentRequired[StatusCode::PAYMENT_REQUIRED],
        Forbidden[StatusCode::FORBIDDEN],
        NotFound[StatusCode::NOT_FOUND],
        MethodNotAllowed[StatusCode::METHOD_NOT_ALLOWED],
        NotAcceptable[StatusCode::NOT_ACCEPTABLE],
        ProxyAuthenticationRequired[StatusCode::PROXY_AUTHENTICATION_REQUIRED],
        RequestTimeout[StatusCode::REQUEST_TIMEOUT],
        Conflict[StatusCode::CONFLICT],
        Gone[StatusCode::GONE],
        LengthRequired[StatusCode::LENGTH_REQUIRED],
        PayloadTooLarge[StatusCode::PAYLOAD_TOO_LARGE],
        UriTooLong[StatusCode::URI_TOO_LONG],
        UnsupportedMediaType[StatusCode::UNSUPPORTED_MEDIA_TYPE],
        RangeNotSatisfiable[StatusCode::RANGE_NOT_SATISFIABLE],
        MisdirectedRequest[StatusCode::MISDIRECTED_REQUEST],
        UnprocessableEntity[StatusCode::UNPROCESSABLE_ENTITY],
        Locked[StatusCode::LOCKED],
        FailedDependency[StatusCode::FAILED_DEPENDENCY],
        UpgradeRequired[StatusCode::UPGRADE_REQUIRED],
        PreconditionFailed[StatusCode::PRECONDITION_FAILED],
        PreconditionRequired[StatusCode::PRECONDITION_REQUIRED],
        TooManyRequests[StatusCode::TOO_MANY_REQUESTS],
        RequestHeaderFieldsTooLarge[StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE],
        UnavailableForLegalReasons[StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS],
        ExpectationFailed[StatusCode::EXPECTATION_FAILED],
        InternalServerError[StatusCode::INTERNAL_SERVER_ERROR],
        NotImplemented[StatusCode::NOT_IMPLEMENTED],
        BadGateway[StatusCode::BAD_GATEWAY],
        ServiceUnavailable[StatusCode::SERVICE_UNAVAILABLE],
        GatewayTimeout[StatusCode::GATEWAY_TIMEOUT],
        HttpVersionNotSupported[StatusCode::HTTP_VERSION_NOT_SUPPORTED],
        VariantAlsoNegotiates[StatusCode::VARIANT_ALSO_NEGOTIATES],
        InsufficientStorage[StatusCode::INSUFFICIENT_STORAGE],
        LoopDetected[StatusCode::LOOP_DETECTED],
        NotExtended[StatusCode::NOT_EXTENDED],
        NetworkAuthenticationRequired[StatusCode::NETWORK_AUTHENTICATION_REQUIRED],
    ];
}

impl<T: fmt::Debug> fmt::Debug for InternalError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for InternalError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(f)
    }
}

impl<T> ResponseError for InternalError<T>
where
    T: fmt::Debug + fmt::Display + 'static,
{
    fn status_code(&self) -> StatusCode {
        self.status
    }
}
