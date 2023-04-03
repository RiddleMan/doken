use anyhow::Result;
use std::process::Command;

pub fn open_authorization_url(url: &str, callback_url: &str) -> Result<()> {
    log::debug!(
        "Using `{}` url to initiate user session. Callback url is: {}",
        url,
        callback_url
    );

    log::debug!("Opening a browser with {url} ...");
    let status = Command::new("open").arg(url).status()?;

    if !status.success() {
        panic!("Url couldn't be opened.")
    }

    Ok(())
}
