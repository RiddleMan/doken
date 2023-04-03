use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct OpenIDProviderMetadata {
    token_endpoint: String,

    authorization_endpoint: String,
}

pub async fn get_endpoints_from_discovery_url(discovery_url: String) -> Result<(String, String)> {
    let result = reqwest::get(discovery_url.to_owned())
        .await
        .context("Couldn't reach out to provided `--discovery-url`")?
        .error_for_status()
        .context("Failed during OIDC discovery call")?
        .json::<OpenIDProviderMetadata>()
        .await
        .context("Couldn't process json given by `--discovery-url`")?;

    Ok((result.token_endpoint, result.authorization_endpoint))
}
