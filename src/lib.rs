#![allow(clippy::too_many_arguments)]
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;

use rquest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    multipart,
    redirect::Policy,
    Impersonate, ImpersonateOS, Method,
};

mod impersonate;
use impersonate::{ImpersonateFromStr, ImpersonateOSFromStr};
mod response;
use response::Response;

#[napi]
pub enum HttpMethod {
    GET,
    HEAD,
    OPTIONS,
    DELETE,
    POST,
    PUT,
    PATCH,
}

impl HttpMethod {
    pub fn to_rquest(&self) -> Method {
        match self {
            HttpMethod::GET => Method::GET,
            HttpMethod::HEAD => Method::HEAD,
            HttpMethod::OPTIONS => Method::OPTIONS,
            HttpMethod::DELETE => Method::DELETE,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::PATCH => Method::PATCH,
        }
    }
}

#[napi]
pub struct Client {
    client: Arc<Mutex<rquest::Client>>,
    headers: Option<HashMap<String, String>>,
    auth: Option<(String, Option<String>)>,
    auth_bearer: Option<String>,
    params: Option<HashMap<String, String>>,
    timeout: Option<f64>,
}

#[napi(object)]
pub struct ClientConfig {
    pub auth: Option<Vec<String>>,
    pub auth_bearer: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub cookie_store: Option<bool>,
    pub referer: Option<bool>,
    pub proxy: Option<String>,
    pub timeout: Option<f64>,
    pub impersonate: Option<String>,
    pub impersonate_os: Option<String>,
    pub follow_redirects: Option<bool>,
    pub max_redirects: Option<u32>,
    pub verify: Option<bool>,
    pub ca_cert_file: Option<String>,
    pub https_only: Option<bool>,
    pub http2_only: Option<bool>,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new(config: Option<ClientConfig>) -> Result<Self> {
        let config = config.unwrap_or(ClientConfig {
            auth: None, auth_bearer: None, headers: None, cookie_store: None, referer: None,
            proxy: None, timeout: None, impersonate: None, impersonate_os: None,
            follow_redirects: None, max_redirects: None, verify: None, ca_cert_file: None,
            https_only: None, http2_only: None
        });

        let auth = if let Some(auth_vec) = &config.auth {
            if !auth_vec.is_empty() {
                let user = auth_vec[0].clone();
                let pass = if auth_vec.len() > 1 { Some(auth_vec[1].clone()) } else { None };
                Some((user, pass))
            } else {
                None
            }
        } else {
            None
        };

        let mut client_builder = rquest::Client::builder();

        if let Some(impersonate) = &config.impersonate {
            let imp = Impersonate::from_str(impersonate).map_err(|e| Error::from_reason(e.to_string()))?;
            let imp_os = if let Some(impersonate_os) = &config.impersonate_os {
                ImpersonateOS::from_str(impersonate_os).map_err(|e| Error::from_reason(e.to_string()))?
            } else {
                ImpersonateOS::default()
            };
            let impersonate_builder = Impersonate::builder()
                .impersonate(imp)
                .impersonate_os(imp_os)
                .build();
            client_builder = client_builder.impersonate(impersonate_builder);
        }

        if let Some(h) = &config.headers {
            let mut header_map = HeaderMap::new();
            for (k, v) in h {
                if let (Ok(key), Ok(val)) = (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(v)) {
                    header_map.insert(key, val);
                }
            }
            client_builder = client_builder.default_headers(header_map);
        };

        if config.cookie_store.unwrap_or(true) {
            client_builder = client_builder.cookie_store(true);
        }

        if config.referer.unwrap_or(true) {
            client_builder = client_builder.referer(true);
        }

        let proxy_url = config.proxy.or_else(|| std::env::var("PRIMP_PROXY").ok());
        if let Some(p) = proxy_url {
             let rproxy = rquest::Proxy::all(&p).map_err(|e| Error::from_reason(e.to_string()))?;
             client_builder = client_builder.proxy(rproxy);
        }

        if let Some(seconds) = config.timeout {
            client_builder = client_builder.timeout(Duration::from_secs_f64(seconds));
        }

        if config.follow_redirects.unwrap_or(true) {
            client_builder = client_builder.redirect(Policy::limited(config.max_redirects.unwrap_or(20) as usize));
        } else {
            client_builder = client_builder.redirect(Policy::none());
        }

        if let Some(ca_bundle_path) = &config.ca_cert_file {
            std::env::set_var("PRIMP_CA_BUNDLE", ca_bundle_path);
        }

        if config.verify.unwrap_or(true) {
        } else {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        if let Some(true) = config.https_only {
            client_builder = client_builder.https_only(true);
        }

        if let Some(true) = config.http2_only {
            client_builder = client_builder.http2_only();
        }

        let client = client_builder.build().map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Client {
            client: Arc::new(Mutex::new(client)),
            headers: config.headers,
            auth,
            auth_bearer: config.auth_bearer,
            params: None,
            timeout: config.timeout,
        })
    }

    #[napi]
    pub async fn request(
        &self,
        method: HttpMethod,
        url: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        cookies: Option<HashMap<String, String>>,
        content: Option<Buffer>,
        data: Option<HashMap<String, String>>,
        json: Option<serde_json::Value>,
        files: Option<HashMap<String, String>>,
        auth: Option<Vec<String>>,
        auth_bearer: Option<String>,
        timeout: Option<f64>,
    ) -> Result<Response> {
        let mut file_parts = Vec::new();
        if let Some(f) = files {
            for (field_name, file_path) in f {
                 let file_bytes = tokio::fs::read(&file_path).await.map_err(|e| Error::from_reason(format!("Failed to read file {}: {}", file_path, e)))?;
                 file_parts.push((field_name, file_path, file_bytes));
            }
        }

        let request_builder = {
            let client_guard = self.client.lock().unwrap(); 
            let request_method = method.to_rquest();
            let mut request_builder = client_guard.request(request_method, &url);

            if let Some(p) = params {
                request_builder = request_builder.query(&p);
            } else if let Some(p) = &self.params {
                request_builder = request_builder.query(p);
            }

            if let Some(h) = headers {
                let mut header_map = HeaderMap::new();
                 for (k, v) in h {
                    if let (Ok(key), Ok(val)) = (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(&v)) {
                        header_map.insert(key, val);
                    }
                }
                request_builder = request_builder.headers(header_map);
            }

            if let Some(c) = cookies {
                 if let Ok(parsed_url) = rquest::Url::parse(&url) {
                      let mut cookie_values = Vec::new();
                      for (k, v) in c {
                           let s = format!("{}={}", k, v);
                           if let Ok(hv) = HeaderValue::from_str(&s) {
                               cookie_values.push(hv);
                           }
                      }
                      client_guard.set_cookies(&parsed_url, cookie_values);
                 }
            }

            if let Some(b) = content {
                request_builder = request_builder.body(b.to_vec());
            }
            else if let Some(d) = data {
                 request_builder = request_builder.form(&d);
            }
            else if let Some(j) = json {
                 request_builder = request_builder.json(&j);
            }
            else if !file_parts.is_empty() {
                let mut form = multipart::Form::new();
                for (field_name, file_path, file_bytes) in file_parts {
                     let part = multipart::Part::bytes(file_bytes).file_name(file_path);
                     form = form.part(field_name, part);
                }
                request_builder = request_builder.multipart(form);
            }

            let req_auth = if let Some(auth_vec) = auth {
                if !auth_vec.is_empty() {
                     let user = auth_vec[0].clone();
                     let pass = if auth_vec.len() > 1 { Some(auth_vec[1].clone()) } else { None };
                     Some((user, pass))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((u, p)) = req_auth.or(self.auth.clone()) {
                 request_builder = request_builder.basic_auth(u, p);
            } else if let Some(token) = auth_bearer.or(self.auth_bearer.clone()) {
                 request_builder = request_builder.bearer_auth(token);
            }

            if let Some(t) = timeout.or(self.timeout) {
                 request_builder = request_builder.timeout(Duration::from_secs_f64(t));
            }
            
            request_builder
        };

        let resp = request_builder.send().await.map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(Response::new(resp, url))
    }
}
