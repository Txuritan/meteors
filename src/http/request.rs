use {
    crate::{
        extensions::Extensions,
        http::{
            self,
            headers::{HeaderName, CONTENT_LENGTH},
            Method, Version,
        },
    },
    std::{borrow::Cow, collections::BTreeMap, io::Read, str::FromStr as _, sync::Arc},
};

#[derive(Debug, PartialEq)]
pub struct HeaderData {
    pub(crate) method: Method,
    pub(crate) url: String,
    pub(crate) query: String,
    pub(crate) query_params: BTreeMap<String, String>,
    #[allow(dead_code)] // just store it as we needed to parse it anyway
    pub(crate) version: Version,
    pub(crate) headers: BTreeMap<HeaderName, String>,
}

pub struct HttpRequest {
    pub(crate) header_data: HeaderData,
    pub(crate) body: Vec<u8>,

    pub(crate) params: BTreeMap<String, String>,

    pub(crate) data: Arc<Extensions>,

    pub(crate) extensions: Extensions,
}

impl HttpRequest {
    pub(crate) fn from_parts(
        header_data: HeaderData,
        body: Vec<u8>,
        params: BTreeMap<String, String>,
        data: Arc<Extensions>,
    ) -> Self {
        Self {
            header_data,
            body,
            params,
            data,
            extensions: Extensions::new(),
        }
    }

    #[cfg(fuzzing)]
    pub fn parse_reader<R>(reader: &mut R) -> Result<(HeaderData, Vec<u8>), http::HttpError>
    where
        R: Read,
    {
        Self::parse_reader_inner(reader)
    }

    #[cfg(not(fuzzing))]
    pub(crate) fn parse_reader<R>(reader: &mut R) -> Result<(HeaderData, Vec<u8>), http::HttpError>
    where
        R: Read,
    {
        Self::parse_reader_inner(reader)
    }

    fn parse_reader_inner<R>(reader: &mut R) -> Result<(HeaderData, Vec<u8>), http::HttpError>
    where
        R: Read,
    {
        struct State {
            data: Vec<u8>,
            total_read: usize,
            amount_read: usize,
            read_buffer: [u8; BUFFER_SIZE],
        }

        fn double_newline(bytes: &[u8]) -> bool {
            bytes == &b"\r\n\r\n"[..]
        }

        const BUFFER_SIZE: usize = 512;
        const MAX_BYTES: usize = 1028 * 8;

        let mut state = State {
            data: Vec::with_capacity(512),
            total_read: 0,
            amount_read: BUFFER_SIZE,
            read_buffer: [0; BUFFER_SIZE],
        };

        loop {
            if state.amount_read == 0 {
                break;
            }

            state.amount_read = reader.read(&mut state.read_buffer)?;

            if state.amount_read == 0 {
                break;
            }

            (state.total_read) += state.amount_read;

            state
                .data
                .extend_from_slice(&state.read_buffer[..state.amount_read]);

            (state.read_buffer) = [0; BUFFER_SIZE];

            if state.data.windows(4).any(double_newline) {
                break;
            }

            if state.total_read >= MAX_BYTES {
                break;
            }
        }

        let header_bytes = if let Some(i) = state.data.windows(4).position(double_newline) {
            &state.data[..(i + 2)]
        } else {
            &state.data[..]
        };

        let header_str = String::from_utf8_lossy(header_bytes);

        let header_data = HttpRequest::parse_header(header_str.as_ref())?;

        let (header_data, body) = if let Some(header) = header_data.headers.get(&CONTENT_LENGTH) {
            let amount_of_bytes = header.trim().parse::<usize>()?;

            let left_to_read = MAX_BYTES.saturating_sub(state.total_read);

            if amount_of_bytes >= left_to_read {
                (header_data, vec![])
            } else {
                let mut body = Vec::with_capacity(left_to_read.min(amount_of_bytes));

                reader
                    .by_ref()
                    .take(amount_of_bytes as u64)
                    .read_to_end(&mut body)?;

                (header_data, Vec::from(&body[..]))
            }
        } else {
            (header_data, vec![])
        };

        Ok((header_data, body))
    }

    #[cfg(fuzzing)]
    pub fn fuzz_parse_header(headers: &str) -> Result<HeaderData, http::HttpError> {
        Self::parse_header_inner(headers)
    }

    #[cfg(not(fuzzing))]
    pub(crate) fn parse_header(headers: &str) -> Result<HeaderData, http::HttpError> {
        Self::parse_header_inner(headers)
    }

    fn parse_header_inner(headers: &str) -> Result<HeaderData, http::HttpError> {
        let mut lines = headers.lines();

        let meta = lines.next().ok_or(http::HttpError::ParseMissingMeta)?;

        let (method, url, query, query_params, version) = {
            let mut meta_parts = meta.split(' ').filter(|part| !part.is_empty());

            let method = Method::from_str(
                meta_parts
                    .next()
                    .ok_or(http::HttpError::ParseMetaMissingMethod)?
                    .trim(),
            )?;

            let url = meta_parts
                .next()
                .ok_or(http::HttpError::ParseMetaMissingUrl)?
                .trim();

            let (url, query) = url.split_at(url.find('?').unwrap_or_else(|| url.len()));

            let query_params = form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<BTreeMap<_, _>>();

            let version = Version::from_str(
                meta_parts
                    .next()
                    .ok_or(http::HttpError::ParseMetaMissingVersion)?
                    .trim(),
            )?;

            (
                method,
                url.to_string(),
                query.to_string(),
                query_params,
                version,
            )
        };

        let headers = {
            let mut headers: BTreeMap<HeaderName, String> = BTreeMap::new();

            for header in &mut lines {
                if header.is_empty() {
                    break;
                }

                if let Some(idx) = header.find(':') {
                    let (key, value) = header.split_at(idx);

                    headers.insert(
                        HeaderName(Cow::Owned(key.trim().to_string())),
                        value.trim_start_matches(": ").trim().to_string(),
                    );
                }
            }

            headers
        };

        Ok(HeaderData {
            method,
            url,
            query,
            query_params,
            version,
            headers,
        })
    }

    pub const fn ext(&self) -> &Extensions {
        &self.extensions
    }

    pub const fn ext_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub fn url(&self) -> &str {
        &self.header_data.url
    }

    pub const fn version(&self) -> Version {
        self.header_data.version
    }

    pub const fn method(&self) -> Method {
        self.header_data.method
    }
}

#[cfg(test)]
mod test_parse {
    use {
        super::*,
        crate::http::headers::{ACCEPT, HOST, USER_AGENT},
    };

    // most are ripped from https://github.com/nodejs/http-parser/blob/main/test.c

    macro http_assert($name:ident, $exp:expr, $got:expr,) {
        #[test]
        fn $name() {
            assert_eq!($exp, $got);
        }
    }

    macro request(
        [ $method:expr, $url:expr, $query:expr, $version:expr ]
        $( { $header:expr => $value:expr } )*
        [ $( $body:expr )* ]
    ) {
        (
            HeaderData {
                method: $method,
                url: $url.to_string(),
                query: $query.to_string(),
                query_params: BTreeMap::new(),
                version: $version,
                headers: {
                    let mut temp = BTreeMap::new();

                    $( temp.insert($header, $value.to_string()); )*

                    temp
                },
            },
            concat!($( $body ),*).as_bytes().to_vec(),
        )
    }

    macro raw($( $raw:expr )*) {
        HttpRequest::parse_reader(&mut std::io::Cursor::new(
            concat!($( $raw ),*).as_bytes(),
        )).unwrap()
    }

    http_assert!(
        test_curl_get,
        request!(
            [ Method::Get, "/test", "", Version::Http11 ]
            { USER_AGENT => "curl/7.18.0 (i486-pc-linux-gnu) libcurl/7.18.0 OpenSSL/0.9.8g zlib/1.2.3.3 libidn/1.1" }
            { HOST => "0.0.0.0=5000" }
            { ACCEPT => "*/*" }
            []
        ),
        raw!(
            "GET /test HTTP/1.1\r\n"
            "User-Agent: curl/7.18.0 (i486-pc-linux-gnu) libcurl/7.18.0 OpenSSL/0.9.8g zlib/1.2.3.3 libidn/1.1\r\n"
            "Host: 0.0.0.0=5000\r\n"
            "Accept: */*\r\n"
            "\r\n"
        ),
    );
}
