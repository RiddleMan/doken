use std::env;

use clap::error::ErrorKind;
use clap::{ArgGroup, Command, CommandFactory, Parser};
use dotenv::dotenv;

use crate::config_file::ConfigFile;
use crate::grant::Grant;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
#[clap(group(
    ArgGroup::new("oauth2")
        .multiple(true)
        .args(["token_url", "authorization_url"])
        .conflicts_with("oidc")
))]
#[clap(group(
    ArgGroup::new("oidc")
        .arg("discovery_url")
        .conflicts_with("oauth2")
))]
pub struct Arguments {
    /// Authentication Grant
    #[clap(long, value_enum, default_value_t = Grant::AuthorizationCodeWithPkce, env = "DOKEN_GRANT")]
    pub grant: Grant,

    /// OAuth 2.0 token exchange url
    #[clap(long, env = "DOKEN_TOKEN_URL")]
    pub token_url: Option<String>,

    /// OAuth 2.0 authorization initiation url
    #[clap(long, env = "DOKEN_AUTHORIZATION_URL")]
    pub authorization_url: Option<String>,

    /// OpenID Connect discovery url
    #[clap(long, env = "DOKEN_DISCOVERY_URL")]
    pub discovery_url: Option<String>,

    /// Callback URL that's been set for your application
    #[clap(long, env = "DOKEN_CALLBACK_URL")]
    pub callback_url: Option<String>,

    /// OAuth 2.0 Client Identifier <https://www.rfc-editor.org/rfc/rfc6749#section-2.2>
    #[clap(long, env = "DOKEN_CLIENT_ID")]
    pub client_id: String,

    /// OAuth 2.0 Client Secret. Please use `--client-secret-stdin`, because it's not get stored in a shell history.  <https://www.rfc-editor.org/rfc/rfc6749#section-2.3.1>
    #[clap(long, env = "DOKEN_CLIENT_SECRET")]
    pub client_secret: Option<String>,

    /// OAuth 2.0 Client Secret from standard input <https://www.rfc-editor.org/rfc/rfc6749#section-2.3.1>
    #[clap(long, action, default_value_t = false)]
    pub client_secret_stdin: bool,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's username <https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2>
    #[clap(short, long, env = "DOKEN_USERNAME")]
    pub username: Option<String>,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's password <https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2>
    #[clap(short, long, env = "DOKEN_PASSWORD")]
    pub password: Option<String>,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's password from standard input <https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2>
    #[clap(long, action, default_value_t = false)]
    pub password_stdin: bool,

    /// OAuth 2.0 Scope <https://www.rfc-editor.org/rfc/rfc6749#section-3.3>
    #[clap(long, default_value = "offline_access", env = "DOKEN_SCOPE")]
    pub scope: String,

    /// OpenID Connect requested aud
    #[clap(long, env = "DOKEN_AUDIENCE")]
    pub audience: Option<String>,

    /// Authorization Code, Authorization Code with PKCE and Implicit Grants' timeout,
    #[clap(short, long, default_value_t = 30_000, env = "DOKEN_TIMEOUT")]
    pub timeout: u64,

    /// When turned on ignores the state file and continues with a fresh flow
    #[clap(short, long, action, default_value_t = false)]
    pub force: bool,

    /// Add diagnostics info
    #[clap(short, long, action, default_value_t = false)]
    pub debug: bool,

    /// Profile defined in ~/.doken/config.toml file
    #[clap(long)]
    pub profile: Option<String>,
}

impl Default for Arguments {
    fn default() -> Self {
        Self {
            grant: Grant::AuthorizationCodeWithPkce,
            token_url: Default::default(),
            authorization_url: Default::default(),
            discovery_url: Default::default(),
            callback_url: Default::default(),
            client_id: Default::default(),
            client_secret: Default::default(),
            client_secret_stdin: Default::default(),
            username: Default::default(),
            password: Default::default(),
            password_stdin: Default::default(),
            scope: Default::default(),
            audience: Default::default(),
            timeout: 30_000,
            force: Default::default(),
            debug: Default::default(),
            profile: Default::default(),
        }
    }
}

pub struct Args;

// TODO: match green color as the rest of clap messages
impl Args {
    fn assert_urls_for_authorization_grants(args: &Arguments) {
        let mut cmd: Command = Arguments::command();

        if args.token_url.is_none()
            && args.authorization_url.is_none()
            && args.discovery_url.is_none()
        {
            cmd.error(
                ErrorKind::MissingRequiredArgument,
                "<--token-url, --authorization-url|--discovery-url> arguments have to be provided",
            )
            .exit();
        }

        if args.callback_url.is_none() {
            cmd.error(
                ErrorKind::MissingRequiredArgument,
                "--callback-url argument have to be provided",
            )
            .exit();
        }
    }

    fn assert_grant_specific_arguments(args: &Arguments) {
        let mut cmd: Command = Arguments::command();

        match args.grant {
            Grant::AuthorizationCodeWithPkce => {
                Self::assert_urls_for_authorization_grants(args);
            }
            Grant::AuthorizationCode => {
                Self::assert_urls_for_authorization_grants(args);
            }
            Grant::ResourceOwnerPasswordClientCredentials => {
                if args.token_url.is_none() && args.discovery_url.is_none() {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "<--token-url|--discovery-url> arguments have to be provided",
                    )
                    .exit();
                }

                if args.client_secret.is_none() && !args.client_secret_stdin {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "--client-secret or --client-secret-stdin is required while used with `client-credentials` grant.",
                    )
                        .exit();
                }

                if args.username.is_none() {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "--username is required while used with `resource-owner-password-client-credentials` grant.",
                    )
                        .exit();
                }

                if args.password.is_none() && !args.password_stdin {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "--password or --password-stdin is required while used with `resource-owner-password-client-credentials` grant.",
                    )
                        .exit();
                }
            }
            Grant::ClientCredentials => {
                if args.token_url.is_none() && args.discovery_url.is_none() {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "<--token-url|--discovery-url> arguments have to be provided",
                    )
                    .exit();
                }

                if args.client_secret.is_none() && !args.client_secret_stdin {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "--client-secret or --client-secret-stdin is required while used with `client-credentials` grant.",
                    )
                        .exit();
                }
            }
            Grant::Implicit => {
                if args.token_url.is_some() {
                    cmd.error(
                        ErrorKind::ArgumentConflict,
                        "--token-url cannot be used with:\n\t--grant implicit",
                    )
                    .exit();
                }

                if args.authorization_url.is_none() && args.discovery_url.is_none() {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "<--authorization-url|--discovery-url> arguments have to be provided",
                    )
                    .exit();
                }

                if args.callback_url.is_none() {
                    cmd.error(
                        ErrorKind::MissingRequiredArgument,
                        "--callback-url argument have to be provided",
                    )
                    .exit();
                }
            }
        }
    }

    fn parse_client_secret(mut args: Arguments) -> Arguments {
        if args.client_secret.is_some() && std::env::var("DOKEN_CLIENT_SECRET").is_err() {
            eprintln!("Please use `--client-secret-stdin` as a more secure variant.");
        }

        if args.client_secret_stdin {
            args.client_secret = Some(rpassword::prompt_password("Client Secret: ").unwrap());
        }

        args
    }

    fn parse_password(mut args: Arguments) -> Arguments {
        if args.password.is_some() && std::env::var("DOKEN_PASSWORD").is_err() {
            eprintln!("Please use `--password-stdin` as a more secure variant.");
        }

        if args.password_stdin {
            args.password = Some(rpassword::prompt_password("Password: ").unwrap());
        }

        args
    }

    async fn apply_profile() {
        let mut cmd: Command = Arguments::command();
        let args: Vec<String> = env::args().collect();
        let profile = match args.iter().position(|arg| arg.eq("--profile")) {
            Some(profile_pos) => args.get(profile_pos + 1).cloned(),
            None => None,
        };

        let config = ConfigFile::new().apply_profile(profile.clone()).await;

        if config.is_err() {
            cmd.error(
                ErrorKind::InvalidValue,
                format!(
                    "--profile `{}` definition cannot be found in ~/.doken/config.toml",
                    profile.unwrap()
                ),
            )
            .exit();
        }
    }

    pub async fn parse() -> Arguments {
        log::debug!("Parsing application arguments...");
        if dotenv().is_ok() {
            log::debug!(".env file found");
        } else {
            log::debug!(".env file not found. skipping...");
        }

        Self::apply_profile().await;

        let args = Arguments::parse();
        Self::assert_grant_specific_arguments(&args);
        let mut args = Self::parse_client_secret(args);
        args = Self::parse_password(args);

        log::debug!("Argument parsing done");
        log::debug!("Running with arguments: {args:#?}");

        args
    }
}
