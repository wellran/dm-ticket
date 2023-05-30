use anyhow::Result;
use dm_ticket::{client::LoginClient, models::qrcode};
use dotenv::dotenv;

use log::{debug, error, info};
use std::{
    env,
    io::{self, Write},
    time::Duration,
};

pub struct DmLogin {
    client: LoginClient,
}

impl DmLogin {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: LoginClient::new().await?,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let qrcode_data = match self.client.generate_qrcode().await {
            Ok(data) => {
                debug!("Get qrcode data:{:?}", data);
                data
            }
            Err(e) => {
                error!("Fail to get qrcode data, error:{:?}", e);
                return Err(e);
            }
        };

        println!("{:?}", qrcode_data);

        let qrcode = match self.client.get_qrcode(qrcode_data.code_content).await {
            Ok(code) => {
                debug!("success to get qrcode!");
                code
            }
            Err(e) => {
                error!("Fail to get qrcode, error:{:?}", e);
                return Err(e);
            }
        };

        info!("请使用大麦APP扫码二维码登录:\n");
        println!("{}", qrcode.to_str());

        let t = qrcode_data.t.clone();
        let ck = qrcode_data.ck.clone();
        //
        // "NEW"

        // println!("{:?}", ck);
        // return Ok(());
        for _ in 0..60 {
            let qrcode_scan_status = self.client.get_login_result(t.clone(), ck.clone()).await?;

            match qrcode_scan_status.qrcode_status.as_str() {
                "NEW" => {
                    print!("\r\t等待扫码...");
                    let _ = io::stdout().flush();
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                "SCANED" => {
                    print!("\r\t已扫码, 等待确认登录...");
                    let _ = io::stdout().flush();
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                "CONFIRMED" => {
                    print!("\r\t已确认, 登录成功...");
                    println!("{:?}", qrcode_scan_status);
                    break;
                }
                "EXPIRED" => {
                    print!("\r\t二维码已过期...");
                    break;
                }
                _ => {
                    error!("未知状态:{:?}", qrcode_scan_status);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("TOKEN_SERVER_URL").is_err() {
        env::set_var("TOKEN_SERVER_URL", "http://127.0.0.1:8080/");
    }

    if env::var("QRCODE_PATH").is_err() {
        env::set_var("QRCODE_PATH", "./qrcode.png");
    }

    pretty_env_logger::init();

    let app = DmLogin::new().await?;
    app.run().await?;

    Ok(())
}
