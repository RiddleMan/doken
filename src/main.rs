#![deny(warnings)]

use doken::args::Args;
use doken::auth_browser::auth_browser::AuthBrowser;
use doken::get_token;
use std::env;
use std::process::exit;
use anyhow::Result;

fn enable_debug_via_args() {
    let has_debug_flag = env::args().any(|s| s.eq("--debug") || s.eq("-d"));

    if env::var("RUST_LOG").is_err() && has_debug_flag {
        env::set_var("RUST_LOG", "debug")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_debug_via_args();
    env_logger::init();

    let args = Args::parse().await;

    let mut auth_browser = AuthBrowser::new(false).await?;
    println!("{}", get_token(args, &mut auth_browser).await?);
    exit(0);
}
