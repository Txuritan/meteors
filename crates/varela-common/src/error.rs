pub struct Error {
    inner: Box<dyn std::error::Error + 'static>,
}

impl Error {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }

    fn display_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error {
            inner: box err,
        }
    }
}

pub trait Context<C> {
    type WithContext;

    fn context(self, ctx: C) -> Self::WithContext;

    fn with_context<F>(self, f: F) -> Self::WithContext
    where
        F: FnOnce() -> C;
}

impl<T, E, C> Context<C> for Result<T, E> {
    type WithContext = Result<T, ErrorContext<E, C>>;

    fn context(self, ctx: C) -> Self::WithContext {
        self.with_context(|| ctx)
    }

    fn with_context<F>(self, f: F) -> Self::WithContext
    where
        F: FnOnce() -> C,
    {
        self.map_err(|err| ErrorContext {
            error: err,
            context: f(),
        })
    }
}

#[derive(Debug)]
pub struct ErrorContext<E, C> {
    error: E,
    context: C,
}

impl<E, C> std::fmt::Display for ErrorContext<E, C>
where
    E: std::fmt::Display,
    C: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.error, self.context)
    }
}

impl<E, C> std::error::Error for ErrorContext<E, C>
where
    E: std::error::Error,
    C: std::fmt::Display + std::fmt::Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}
