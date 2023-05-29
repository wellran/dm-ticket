use anyhow::Result;
use dm_ticket::client::LoginClient;
use dotenv::dotenv;
use dm_ticket::models::qrcode::QrcodeGenerateParams;
use std::env;


#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("TOKEN_SERVER_URL").is_err() {
        env::set_var("TOKEN_SERVER_URL", "http://127.0.0.1:8080/");
    }

    // pretty_env_logger::init();

    let login_client = LoginClient::new().await?;
    println!("login_client:{:?}", login_client);
    let data = login_client.generate_qrcode().await?;
    println!("{:?}", data);

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("TOKEN_SERVER_URL").is_err() {
        env::set_var("TOKEN_SERVER_URL", "http://127.0.0.1:8080/");
    }

    pretty_env_logger::init();

    Ok(())
}
