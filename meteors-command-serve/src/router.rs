use {
    chrono::Duration,
    common::prelude::*,
    path_tree::PathTree,
    qstring::QString,
    std::{borrow::Cow, collections::BTreeMap, io::Cursor, sync::Arc, time::Instant},
};

pub use tiny_http::{Header, HeaderField, Request, StatusCode};

pub static PAGE_404: &str = r#"<!DOCTYPE html><html><head><title>404 | local archive</title><style>*{transition:all .6s}html{height:100%}body{font-family:sans-serif;color:#888;margin:0}#main{display:table;width:100%;height:100vh;text-align:center}.fof{display:table-cell;vertical-align:middle}.fof h1{font-size:50px;display:inline-block;padding-right:12px;animation:type .5s alternate infinite}@keyframes type{from{box-shadow:inset -3px 0 0 #888}to{box-shadow:inset -3px 0 0 transparent}}</style></head><body><div id="main"><div class="fof"><h1>Error 404</h1></div></div></body></html>"#;
pub static PAGE_503: &str = r#"<!DOCTYPE html><html><head><title>503 | local archive</title><style>*{transition:all .6s}html{height:100%}body{font-family:sans-serif;color:#888;margin:0}#main{display:table;width:100%;height:100vh;text-align:center}.fof{display:table-cell;vertical-align:middle}.fof h1{font-size:50px;display:inline-block;padding-right:12px;animation:type .5s alternate infinite}@keyframes type{from{box-shadow:inset -3px 0 0 #888}to{box-shadow:inset -3px 0 0 transparent}}</style></head><body><div id="main"><div class="fof"><h1>Error 503</h1></div></div></body></html>"#;

macro_rules! res {
    (404) => {
        Response::from_string(PAGE_404)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
            .with_status_code(404)
    };
    (503) => {
        Response::from_string(PAGE_503)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
            .with_status_code(503)
    };
}

pub type Response = tiny_http::Response<Cursor<Vec<u8>>>;

pub struct Context<'s, S> {
    state: Arc<S>,
    params: Vec<(&'s str, &'s str)>,
    query: Vec<(Cow<'s, str>, Cow<'s, str>)>,
    raw_query: &'s str,

    pub headers: &'s [Header],
}

impl<'s, S> Context<'s, S> {
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn param(&self, key: &str) -> Option<&'s str> {
        self.params
            .iter()
            .find_map(|(k, v)| (*k == key).then(|| *v))
    }

    pub fn query(&self, key: &str) -> Option<Cow<'s, str>> {
        self.query
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, value)| value.clone())
    }

    pub fn rebuild_query(&self) -> Cow<'static, str> {
        if self.raw_query.is_empty() {
            Cow::from(String::new())
        } else {
            let mut parsed = QString::from(self.raw_query).into_pairs();

            parsed.retain(|(k, _)| k != "search");

            Cow::from(format!("?{}", QString::new(parsed)))
        }
    }
}

pub trait Route<'r, S>: 'static {
    fn call(&'r self, ctx: &'r Context<'r, S>) -> Result<Response>;
}

pub struct Boxed<S>(Box<dyn (for<'r> Route<'r, S>) + Send + Sync>);

impl<'r, S, T> Route<'r, S> for T
where
    S: 'r,
    T: 'static + Fn(&'r Context<'r, S>) -> Result<Response>,
{
    fn call(&'r self, ctx: &'r Context<'r, S>) -> Result<Response> {
        (self)(ctx)
    }
}

impl<'r, S> Route<'r, S> for Boxed<S>
where
    S: 'static,
{
    fn call(&'r self, ctx: &'r Context<'r, S>) -> Result<Response> {
        self.0.call(ctx)
    }
}

pub struct Handler<S> {
    method: Method,
    route: Boxed<S>,
}

pub fn get<R, S>(route: R) -> Handler<S>
where
    R: (for<'r> Route<'r, S>) + Send + Sync,
{
    Handler {
        method: Method::Get,
        route: Boxed(Box::new(route)),
    }
}

#[allow(dead_code)]
pub fn post<R, S>(route: R) -> Handler<S>
where
    R: (for<'r> Route<'r, S>) + Send + Sync,
{
    Handler {
        method: Method::Post,
        route: Boxed(Box::new(route)),
    }
}

pub struct Router<S> {
    tree: BTreeMap<Method, PathTree<Boxed<S>>>,
    state: Arc<S>,
}

impl<S> Router<S> {
    pub fn new(state: S) -> Self {
        Self {
            tree: BTreeMap::new(),
            state: Arc::new(state),
        }
    }

    pub fn on(mut self, path: &str, handler: Handler<S>) -> Self {
        let node = self
            .tree
            .entry(handler.method)
            .or_insert_with(PathTree::new);

        node.insert(path, handler.route);

        self
    }

    pub fn handle(&self, request: Request) -> Result<()>
    where
        S: 'static,
    {
        let earlier = Instant::now();

        let url = request.url();
        let (url, raw_query) = url.split_at(url.find('?').unwrap_or_else(|| url.len()));
        let query = form_urlencoded::parse(raw_query.trim_start_matches('?').as_bytes())
            .collect::<Vec<_>>();

        let method = Method::from(request.method());

        info!(
            "{} {} {}/{} {} {}",
            "+".bright_black(),
            "+".bright_black(),
            "HTTP".bright_yellow(),
            request.http_version(),
            method.to_colored_string(),
            url.bright_purple(),
        );

        let state = Arc::clone(&self.state);

        let response = self
            .tree
            .get(&method)
            .and_then(|tree| tree.find(url))
            .map_or(Ok(res!(404)), |(payload, params)| {
                payload.call(&Context {
                    headers: request.headers(),
                    state,
                    query,
                    params,
                    raw_query,
                })
            })
            .map_err(|err| {
                for cause in err.chain() {
                    error!("  {} {}", "|".bright_black(), cause,);
                }

                res!(503)
            })
            .ignore();

        let dur = Duration::from_std(Instant::now().duration_since(earlier))?;

        info!(
            "{} {} {} {}ms",
            "+".bright_black(),
            "+".bright_black(),
            match response.status_code().0 {
                200 => format!("{}", "200".green()),
                404 => format!("{}", "404".bright_yellow()),
                503 => format!("{}", "200".bright_red()),
                code => format!("{}", code.to_string().bright_blue()),
            },
            dur.num_milliseconds().bright_purple(),
        );

        request.respond(response)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

impl Method {
    fn to_colored_string(self) -> String {
        match self {
            Method::Get => format!("{}", "GET".green()),
            Method::Post => format!("{}", "POST".bright_blue()),
            Method::Put => format!("{}", "PUT".bright_purple()),
            Method::Patch => format!("{}", "PATCH".bright_yellow()),
            Method::Delete => format!("{}", "DELETE".bright_red()),
            Method::Head => "HEAD".to_owned(),
            Method::Connect => "CONNECT".to_owned(),
            Method::Options => "OPTION".to_owned(),
            Method::Trace => "TRACE".to_owned(),
        }
    }
}

impl From<tiny_http::Method> for Method {
    fn from(method: tiny_http::Method) -> Self {
        Method::from(&method)
    }
}

impl From<&tiny_http::Method> for Method {
    fn from(method: &tiny_http::Method) -> Self {
        match method {
            tiny_http::Method::Get | tiny_http::Method::NonStandard(_) => Method::Get,
            tiny_http::Method::Head => Method::Head,
            tiny_http::Method::Post => Method::Post,
            tiny_http::Method::Put => Method::Put,
            tiny_http::Method::Delete => Method::Delete,
            tiny_http::Method::Connect => Method::Connect,
            tiny_http::Method::Options => Method::Options,
            tiny_http::Method::Trace => Method::Trace,
            tiny_http::Method::Patch => Method::Patch,
        }
    }
}

trait ResultExt<T> {
    fn ignore(self) -> T;
}

impl<T> ResultExt<T> for Result<T, T> {
    fn ignore(self) -> T {
        match self {
            Ok(t) | Err(t) => t,
        }
    }
}
