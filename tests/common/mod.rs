pub mod keycloak;

use ::keycloak::{KeycloakAdmin, KeycloakAdminToken};
use anyhow::Result;
use testcontainers::clients;

use self::keycloak::Keycloak;

pub async fn setup_keycloak() -> Result<KeycloakAdmin> {
    let docker = clients::Cli::default();
    let kc_image = Keycloak::default();

    let kc = docker.run(kc_image.to_owned());


    let url = format!("http://localhost:{}", kc.get_host_port_ipv4(8080));

    let client = reqwest::Client::new();
    let admin_token = KeycloakAdminToken::acquire(&url, kc_image.username(), kc_image.password(), &client).await?;


    Ok(KeycloakAdmin::new(&url, admin_token, client))
}
