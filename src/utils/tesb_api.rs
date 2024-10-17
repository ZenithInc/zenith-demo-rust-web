use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const API: &str = "https://apiesb.zhiyoubao.com";

const VERSION: &str = "1.0";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestParams {
    pub park_code: String,
    pub begin_time: String,
    pub end_time: String,
    pub service_code: String,
    pub page_size: u32,
    pub page_index: u32,
}

pub struct RequestParamsBuilder {
    park_code: String,
    begin_time: String,
    end_time: String,
    service_code: String,
    page_size: u32,
    page_index: u32,
}

impl RequestParamsBuilder {
    pub fn builder() -> Self {
        Self {
            park_code: String::new(),
            begin_time: String::new(),
            end_time: String::new(),
            service_code: String::new(),
            page_size: 500,
            page_index: 1,
        }
    }

    pub fn park_code(mut self, pack_code: &str) -> Self {
        self.park_code = pack_code.to_string();
        self
    }

    pub fn begin_time(mut self, begin_time: &str) -> Self {
        self.begin_time = begin_time.to_string();
        self
    }

    pub fn end_time(mut self, end_time: &str) -> Self {
        self.end_time = end_time.to_string();
        self
    }

    pub fn service_code(mut self, service_code: &str) -> Self {
        self.service_code = service_code.to_string();
        self
    }

    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn page_index(mut self, page_index: u32) -> Self {
        self.page_index = page_index;
        self
    }

    pub fn build(self) -> RequestParams {
        RequestParams {
            park_code: self.park_code,
            begin_time: self.begin_time,
            end_time: self.end_time,
            service_code: self.service_code,
            page_size: self.page_size,
            page_index: self.page_index,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub request_id: String,
    pub app_id: String,
    pub biz_content: String,
    pub name: String,
    pub sign: String,
    pub timestamp: String,
    pub version: String,
}

pub struct RequestBodyBuilder {
    request_id: String,
    app_id: String,
    biz_content: String,
    name: String,
    sign: String,
    timestamp: String,
    version: String,
}

impl RequestBodyBuilder {
    pub fn builder() -> Self {
        Self {
            request_id: String::new(),
            app_id: String::new(),
            biz_content: String::new(),
            name: String::new(),
            sign: String::new(),
            timestamp: String::new(),
            version: String::new(),
        }
    }

    pub fn request_id(mut self, request_id: &str) -> Self {
        self.request_id = request_id.to_string();
        self
    }

    pub fn app_id(mut self, app_id: &str) -> Self {
        self.app_id = app_id.to_string();
        self
    }

    pub fn biz_content(mut self, biz_content: &str) -> Self {
        self.biz_content = biz_content.to_string();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn sign(mut self, sign: &str) -> Self {
        self.sign = sign.to_string();
        self
    }

    pub fn timestamp(mut self, timestamp: &str) -> Self {
        self.timestamp = timestamp.to_string();
        self
    }

    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn build(self) -> RequestBody {
        RequestBody {
            request_id: self.request_id,
            app_id: self.app_id,
            biz_content: self.biz_content,
            name: self.name,
            sign: self.sign,
            timestamp: self.timestamp,
            version: self.version,
        }
    }
}

fn calculate_signature(request_body: HashMap<&str, &String>, app_key: &str) -> String {
    let mut sorted_params = BTreeMap::new();
    for (key, value) in request_body {
        sorted_params.insert(key, value.as_str().to_string());
    }

    let mut str = String::new();
    for (key, value) in sorted_params {
        str.push_str(&key);
        str.push_str(&value);
    }
    let str = format!("{}{}{}", app_key, str, app_key);
    let sign = format!("{:x}", md5::compute(str));
    sign.to_uppercase()
}

pub struct Config {
    pub app_id: String,
    pub app_key: String,
    pub timeout: Duration,
}


pub async fn request(path: &str, request_params: &RequestParams, configs: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let biz_content = serde_json::to_string(request_params)?;

    let now = SystemTime::now();
    let timestamp = match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis().to_string(),
        Err(e) => return Err(Box::new(e)),
    };

    let uuid = Uuid::new_v4().to_string();

    let app_id = configs.app_id.to_string();
    let path = path.to_string();
    let version = VERSION.to_string();

    let mut params = HashMap::new();
    params.insert("requestId", &uuid);
    params.insert("appId", &app_id);
    params.insert("bizContent", &biz_content);
    params.insert("name", &path);
    params.insert("version", &version);
    params.insert("timestamp", &timestamp);

    let sign = calculate_signature(params, configs.app_key.as_str());


    let body = RequestBodyBuilder::builder()
        .app_id(&app_id)
        .biz_content(&biz_content)
        .timestamp(&timestamp)
        .name(path.as_str())
        .version(VERSION)
        .request_id(&uuid)
        .sign(&sign)
        .build();

    let client = Client::new();
    let response = client.post(API)
        .timeout(configs.timeout)
        .form(&body)
        .send()
        .await?;

    let response_text = response.text().await?;
    Ok(response_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_request() {
        dotenv::dotenv().ok();
        let pack_code = std::env::var("TESB_PACK_CODE").unwrap();
        let service_code = std::env::var("TESB_SERVICE_CODE").unwrap();
        let app_id = std::env::var("TESB_APP_ID").unwrap();
        let app_key = std::env::var("TESB_APP_KEY").unwrap();

        let request_params = RequestParamsBuilder::builder()
            .park_code(&pack_code)
            .begin_time("2024-10-16 00:00:00")
            .end_time("2024-10-16 03:00:00")
            .service_code(&service_code)
            .page_size(1000)
            .page_index(1)
            .build();

        let configs = Config {
            app_id,
            app_key,
            timeout: Duration::from_secs(10),
        };

        let path = "push.bigdata.java.v2.offlineSaleDetail";
        let result = request(path, &request_params, &configs).await;

        assert!(result.is_ok());
        assert!(!result.unwrap().contains("23000"), "signature error");
    }
}