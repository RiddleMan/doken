use keycloak::types::{RealmRepresentation, UserRepresentation};
use ::keycloak::{KeycloakAdmin, KeycloakAdminToken};
use testcontainers::clients;

use crate::common::keycloak::Keycloak;

mod common;

#[tokio::test]
async fn it_adds_two() {
    env_logger::init();
    let docker = clients::Cli::default();
    let kc_image = Keycloak::default();

    let kc = docker.run(kc_image.to_owned());


    let url = format!("http://localhost:{}", kc.get_host_port_ipv4(8080));

    let client = reqwest::Client::new();
    let admin_token = KeycloakAdminToken::acquire(&url, kc_image.username(), kc_image.password(), &client).await.unwrap();
    let admin = KeycloakAdmin::new(&url, admin_token, client);

    admin
        .post(RealmRepresentation {
            realm: Some("test".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    admin
        .realm_users_post(
            "test",
            UserRepresentation {
                username: Some("user".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let users = admin
        .realm_users_get(
            "test", None, None, None, None, None, None, None, None, None, None, None, None, None,
            None,
        )
        .await
        .unwrap();

    eprintln!("{:?}", users);

    let id = users
        .iter()
        .find(|u| u.username == Some("user".into()))
        .unwrap()
        .id
        .as_ref()
        .unwrap()
        .to_string();

    admin
        .realm_users_with_id_delete("test", id.as_str())
        .await
        .unwrap();

    admin.realm_delete("test").await.unwrap();

    assert_eq!(4, 4);
}
