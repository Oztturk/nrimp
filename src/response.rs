use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::Mutex;
use http_body_util::BodyExt;
use encoding_rs::{Encoding, UTF_8};
use mime::Mime;
use std::collections::HashMap;

#[napi]
pub struct Response {
    headers_map: HashMap<String, String>,
    cookies_map: HashMap<String, String>,
    
    body: Arc<Mutex<Option<rquest::Body>>>,
    
    cached_bytes: Arc<Mutex<Option<Vec<u8>>>>,
    
    #[napi(readonly)]
    pub url: String,
    #[napi(readonly)]
    pub status_code: u16,
    
    encoding: Arc<Mutex<Option<String>>>,
}

#[napi]
impl Response {
    pub(crate) fn new(resp: rquest::Response, url: String) -> Self {
        let status_code = resp.status().as_u16();
        let http_resp: http::Response<rquest::Body> = resp.into();
        let (parts, body) = http_resp.into_parts();
        
        let mut headers_map = HashMap::new();
        for (k, v) in parts.headers.iter() {
            if let Ok(val) = v.to_str() {
                headers_map.insert(k.to_string(), val.to_string());
            }
        }
        
        let mut cookies_map = HashMap::new();
        for v in parts.headers.get_all(http::header::SET_COOKIE) {
             if let Ok(val) = v.to_str() {
                 if let Some((name, value)) = val.split_once('=') {
                     let value = value.split(';').next().unwrap_or("").trim();
                     cookies_map.insert(name.trim().to_string(), value.to_string());
                 }
             }
        }

        Self {
            headers_map,
            cookies_map,
            body: Arc::new(Mutex::new(Some(body))),
            cached_bytes: Arc::new(Mutex::new(None)),
            url,
            status_code,
            encoding: Arc::new(Mutex::new(None)),
        }
    }

    #[napi]
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers_map.clone()
    }

    #[napi]
    pub fn cookies(&self) -> HashMap<String, String> {
        self.cookies_map.clone()
    }

    #[napi]
    pub async fn text(&self) -> Result<String> {
        let bytes = self.get_content_bytes().await?;
        let encoding_name = self.get_encoding().await?;
        let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);
        let (text, _, _) = encoding.decode(&bytes);
        Ok(text.to_string())
    }

    #[napi]
    pub async fn json(&self) -> Result<serde_json::Value> {
        let bytes = self.get_content_bytes().await?;
        let v: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(v)
    }

    #[napi]
    pub async fn content(&self) -> Result<Buffer> {
         let bytes = self.get_content_bytes().await?;
         Ok(bytes.into())
    }
    
    async fn get_content_bytes(&self) -> Result<Vec<u8>> {
        let mut cached = self.cached_bytes.lock().await;
        if let Some(bytes) = &*cached {
            return Ok(bytes.clone());
        }
        
        let mut body_guard = self.body.lock().await;
        if let Some(body) = body_guard.take() {
            let collected = BodyExt::collect(body).await.map_err(|e| Error::from_reason(e.to_string()))?;
            let bytes = collected.to_bytes().to_vec();
            *cached = Some(bytes.clone());
            Ok(bytes)
        } else {
             Err(Error::from_reason("Body already consumed"))
        }
    }
    
    async fn get_encoding(&self) -> Result<String> {
        let mut encoding_guard = self.encoding.lock().await;
        if let Some(enc) = &*encoding_guard {
            return Ok(enc.clone());
        }
        
        let content_type = self.headers_map.get("content-type"); 
        
        let mut enc = "utf-8".to_string();
        if let Some(ct) = content_type {
             if let Ok(mime) = ct.parse::<Mime>() {
                 if let Some(charset) = mime.get_param("charset") {
                     enc = charset.to_string();
                 }
             }
        }
        
        *encoding_guard = Some(enc.clone());
        Ok(enc)
    }
}
