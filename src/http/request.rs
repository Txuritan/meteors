use {
    crate::{
        extensions::Extensions,
        http::{self, Method, Version},
    },
    std::{collections::BTreeMap, str::FromStr as _, sync::Arc},
};

pub(crate) struct HeaderData {
    pub(crate) method: Method,
    pub(crate) url: String,
    pub(crate) query: String,
    pub(crate) query_params: BTreeMap<String, String>,
    #[allow(dead_code)] // just store it as we needed to parse it anyway
    pub(crate) version: Version,
    pub(crate) headers: BTreeMap<String, String>,
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

    pub(crate) fn parse_header(headers: &str) -> Result<HeaderData, http::Error> {
        let mut lines = headers.lines();

        let meta = lines.next().ok_or(http::Error::ParseMissingMeta)?;

        let (method, url, query, query_params, version) = {
            let mut meta_parts = meta.split(' ').filter(|part| !part.is_empty());

            let method = Method::from_str(
                meta_parts
                    .next()
                    .ok_or(http::Error::ParseMetaMissingMethod)?
                    .trim(),
            )?;

            let url = meta_parts
                .next()
                .ok_or(http::Error::ParseMetaMissingUrl)?
                .trim();

            let (url, query) = url.split_at(url.find('?').unwrap_or_else(|| url.len()));

            let query_params = form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<BTreeMap<_, _>>();

            let version = Version::from_str(
                meta_parts
                    .next()
                    .ok_or(http::Error::ParseMetaMissingVersion)?
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
            let mut headers = BTreeMap::new();

            for header in &mut lines {
                if header.is_empty() {
                    break;
                }

                if let Some(idx) = header.find(':') {
                    let (key, value) = header.split_at(idx);

                    headers.insert(key.trim().to_string(), value.trim().to_string());
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

    pub fn ext(&self) -> &Extensions {
        &self.extensions
    }

    pub fn ext_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub fn url(&self) -> &str {
        &self.header_data.url
    }

    pub fn version(&self) -> Version {
        self.header_data.version
    }

    pub fn method(&self) -> Method {
        self.header_data.method
    }
}
