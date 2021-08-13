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

pub(crate) struct HeaderData {
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

    #[cfg(feature = "fuzzing")]
    pub fn fuzz_parse_reader<R>(reader: &mut R) -> Result<(HeaderData, Vec<u8>), http::HttpError>
    where
        R: Read,
    {
        Self::parse_reader(reader)
    }

    pub(crate) fn parse_reader<R>(reader: &mut R) -> Result<(HeaderData, Vec<u8>), http::HttpError>
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
            let amount_of_bytes = header.trim().parse::<u64>()?;

            let mut body = Vec::with_capacity(amount_of_bytes as usize);

            reader
                .by_ref()
                .take(amount_of_bytes)
                .read_to_end(&mut body)?;

            (header_data, Vec::from(&body[..]))
        } else {
            (header_data, vec![])
        };

        Ok((header_data, body))
    }

    #[cfg(feature = "fuzzing")]
    pub fn fuzz_parse_header(headers: &str) -> Result<HeaderData, http::HttpError> {
        Self::parse_header(headers)
    }

    pub(crate) fn parse_header(headers: &str) -> Result<HeaderData, http::HttpError> {
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
                        value.trim().to_string(),
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
