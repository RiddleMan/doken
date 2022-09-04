use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize, Serialize, Debug)]
struct OpenIDProviderMetadata {
    token_endpoint: String,

    authorization_endpoint: String,
}

pub async fn get_endpoints_from_discovery_url(
    discovery_url: String,
) -> Result<(String, String), Box<dyn Error>> {
    let result = reqwest::get(discovery_url.to_owned())
        .await
        .expect("Couldn't reach out to provided `--discovery-url`")
        .json::<OpenIDProviderMetadata>()
        .await
        .expect("Couldn't process json given by `--discovery-url`");

    Ok((result.token_endpoint, result.authorization_endpoint))
}
