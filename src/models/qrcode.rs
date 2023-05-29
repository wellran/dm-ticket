use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};
pub struct QrcodeGenerateParams {}

impl QrcodeGenerateParams {
    pub fn build() -> Result<serde_json::Value>{
        Ok(json!({
            "appName": "damai",
            "fromSite": "18",
            "appEntrance": "damai",
            "_csrf_token": "H1jBVpSogSJEo3r99YTMC3",
            "umidToken": "dfcbaaafeb9cc67321b83fa8c822bdff7fb63f74",
            "isMobile": "false",
            "lang": "zh_CN",
            "returnUrl": "https://passport.damai.cn/dologin.htm?redirectUrl=https%253A%252F%252Fwww.damai.cn%252F&platform=106002",
            "hsiz": "13d2466b0da2c009670131a60e852012",
            "bizParams": "",
            "umidTag": "SERVER",
            "_bx-v": "2.2.3",
        }))
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QrcodeData {
    pub t: u64,

    #[serde(rename = "codeContent")]
    pub code_content: String,

    pub ck: String,

    #[serde(rename = "resultCode")]
    pub result_code: u32,
}