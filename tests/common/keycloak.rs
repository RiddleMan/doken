use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image, ImageArgs};

const NAME: &str = "quay.io/keycloak/keycloak";
const TAG: &str = "23.0.7";

#[derive(Debug, Default, Clone)]
pub struct KeycloakArgs;

impl ImageArgs for KeycloakArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        Box::new(vec!["start-dev".to_owned()].into_iter())
    }
}

#[derive(Debug, Clone)]
pub struct Keycloak {
    env_vars: HashMap<String, String>,
    tag: String,
    username: String,
    password: String,
}

impl Keycloak {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

impl Default for Keycloak {
    fn default() -> Self {
        let mut env_vars = HashMap::new();

        let username = "admin".to_owned();
        let password = "admin".to_owned();

        env_vars.insert("KEYCLOAK_ADMIN".to_owned(), username.to_owned());
        env_vars.insert("KEYCLOAK_ADMIN_PASSWORD".to_owned(), password.to_owned());

        Keycloak {
            env_vars,
            tag: TAG.to_owned(),
            username,
            password,
        }
    }
}

impl Image for Keycloak {
    type Args = KeycloakArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        self.tag.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout(
            "Running the server in development mode. DO NOT use this configuration in production.",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![8080, 8080]
    }
}
