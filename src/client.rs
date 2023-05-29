use std::{env, time::Instant};

use anyhow::Result;
use log::{debug, warn};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::{json, Value};

use crate::models::{ticket::TicketInfoParams, DmRes, DmToken, DmLoginRes};
use crate::models::qrcode::{QrcodeData, QrcodeGenerateParams};

const SUCCESS_CODE: u64 = 200;
const SYSTEM_ERROR_CODE: u16 = 500;

#[derive(Debug)]
pub struct TokenClient {
    pub client: Client,
}

impl TokenClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(Self { client })
    }

    // Get value from api.
    pub async fn get_value(&self, key: &str) -> Result<String> {
        let url = env::var("TOKEN_SERVER_URL").unwrap();

        let params = json!({
            "key": key,
        });

        let data = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let code = data
            .get("code")
            .unwrap_or(&SYSTEM_ERROR_CODE.into())
            .as_u64()
            .unwrap();

        Ok(match code {
            SUCCESS_CODE => {
                let value = data["data"]["value"].as_str().unwrap().to_string();
                debug!("Get {}:{}", key, value);
                value
            }
            _ => {
                warn!("Fail to get {}.", key);
                "".to_string()
            }
        })
    }

    // Get bx ua.
    pub async fn get_bx_ua(&self) -> Result<String> {
        let start = Instant::now();
        let bx_ua = self.get_value("bx_ua").await?;
        debug!("获取bx_ua: {:?}, 花费时间:{:?}", bx_ua, start.elapsed());
        Ok(bx_ua)
    }

    // Get bx token.
    pub async fn get_bx_token(&self) -> Result<String> {
        let start = Instant::now();
        let bx_token = self.get_value("bx_token").await?;
        debug!(
            "获取bx_token: {:?}, 花费时间:{:?}",
            bx_token,
            start.elapsed()
        );
        Ok(bx_token)
    }
}

#[derive(Debug)]
pub struct DmClient {
    pub client: Client,
    pub token_client: TokenClient,
    pub token: DmToken,
    pub bx_token: String,
}

// 获取token
pub async fn get_token(cookie: &str) -> Result<DmToken> {
    let mut headers = HeaderMap::new();
    let url = "https://mtop.damai.cn/";

    headers.append("origin", HeaderValue::from_str(url)?);
    headers.append("referer", HeaderValue::from_str(url)?);
    headers.append("cookie", HeaderValue::from_str(cookie)?);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .http2_prior_knowledge()
        .build()?;

    let mut token = DmToken {
        enc_token: "".to_string(),
        token_with_time: "".to_string(),
        token: "".to_string(),
    };

    let url = "https://mtop.damai.cn/h5/mtop.damai.wireless.search.broadcast.list/1.0/?";
    let params = TicketInfoParams::build()?;
    let response = client.get(url).form(&params).send().await?;

    for cookie in response.cookies() {
        if cookie.name() == "_m_h5_tk" {
            token.token_with_time = cookie.value().to_string();
            token.token = token.token_with_time.split('_').collect::<Vec<_>>()[0].to_string();
        }
        if cookie.name() == "_m_h5_tk_enc" {
            token.enc_token = cookie.value().to_string();
        }
    }
    Ok(token)
}

impl DmClient {
    // 初始化请求客户端
    pub async fn new(cookie: String) -> Result<Self> {
        let token_client = TokenClient::new()?;

        let bx_token = token_client.get_bx_token().await?;

        let token = get_token(&cookie).await?;

        let mut headers = HeaderMap::new();

        let base_url = "https://mtop.damai.cn/";
        headers.append("origin", HeaderValue::from_str(base_url)?);
        headers.append("referer", HeaderValue::from_str(base_url)?);

        headers.append(
            "cookie",
            HeaderValue::from_str(
                format!(
                    "{};_m_h5_tk_enc={};_m_h5_tk={};",
                    &cookie, token.enc_token, token.token_with_time
                )
                .as_str(),
            )?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .http2_prior_knowledge()
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3")
            .use_rustls_tls()
            .build()?;
        Ok(Self {
            client,
            token,
            token_client,
            bx_token,
        })
    }

    // 请求API
    pub async fn request(&self, url: &str, mut params: Value, data: Value) -> Result<DmRes> {
        let s = format!(
            "{}&{}&{}&{}",
            self.token.token,
            params["t"].as_str().unwrap(),
            params["appKey"].as_str().unwrap(),
            serde_json::to_string(&data)?,
        );

        let sign = format!("{:?}", md5::compute(s));

        params["sign"] = sign.into();

        params["bx-umidtoken"] = self.bx_token.clone().into();
        params["bx-ua"] = self.token_client.get_bx_ua().await?.into();

        let form = json!({
            "data": serde_json::to_string(&data)?,
        });

        let response = self
            .client
            .post(url)
            .query(&params)
            .form(&form)
            .send()
            .await?;

        let data = response.json::<DmRes>().await?;

        Ok(data)
    }
}

#[derive(Debug)]
pub struct LoginClient {
    pub token_client: TokenClient,
    pub client: Client,
    pub bx_token: String,
}

impl LoginClient {
    pub async fn new() -> Result<Self> {
        let token_client = TokenClient::new()?;
        let bx_token = token_client.get_bx_token().await?;

        let mut headers = HeaderMap::new();
        headers.append("user-agent", HeaderValue::from_str("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")?);
        headers.append("referer", HeaderValue::from_str("https://ipassport.damai.cn/mini_login.htm?lang=zh_cn&appName=damai&appEntrance=default&styleType=vertical&bizParams=&notLoadSsoView=true&notKeepLogin=false&isMobile=false&showSnsLogin=false&regUrl=https%3A%2F%2Fpassport.damai.cn%2Fregister&plainReturnUrl=https%3A%2F%2Fpassport.damai.cn%2Flogin&returnUrl=https%3A%2F%2Fpassport.damai.cn%2Fdologin.htm%3FredirectUrl%3Dhttps%25253A%25252F%25252Fwww.damai.cn%25252F%26platform%3D106002&rnd=0.6260742856882737")?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .http2_prior_knowledge()
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3")
            .use_rustls_tls()
            .build()?;
        Ok(Self {
            token_client,
            client,
            bx_token,
        })
    }

    pub async fn request(&self, url: &str, mut params: Value) -> Result<DmLoginRes> {

        params["bx-umidtoken"] = self.bx_token.clone().into();
        params["bx-ua"] = self.token_client.get_bx_ua().await?.into();

        let response = self
            .client
            .post(url)
            .query(&params)
            .send()
            .await?;

        let data = response.json::<DmLoginRes>().await?;

        Ok(data)
    }

    pub async fn generate_qrcode(&self) -> Result<QrcodeData> {
        let url = "https://ipassport.damai.cn/newlogin/qrcode/generate.do";
        let res = self.request(url, QrcodeGenerateParams::build()?).await?;
        let data = serde_json::from_value(res.content.data)?;
        Ok(data)
    }

}