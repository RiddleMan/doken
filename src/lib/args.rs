use anyhow::{Context, Result};
use clap::error::ErrorKind;
use clap::{ArgGroup, Command, CommandFactory, Parser, ValueEnum};
use dotenv::dotenv;

#[derive(ValueEnum, Clone, Debug)]
pub enum Grant {
    /// Authorization code with PKCE Grant. More: https://www.rfc-editor.org/rfc/rfc7636
    AuthorizationCodeWithPKCE,
    /// Authorization Code Grant. More: https://www.rfc-editor.org/rfc/rfc6749#section-4.1
    AuthorizationCode,
    /// Implicit Grant. More: https://www.rfc-editor.org/rfc/rfc6749#section-4.2
    Implicit,
    /// Resource Owner Client Credentials Grant. More: https://www.rfc-editor.org/rfc/rfc6749#section-4.3
    ResourceOwnerPasswordClientCredentials,
    /// Client credentials Grant. More: https://www.rfc-editor.org/rfc/rfc6749#section-4.4
    ClientCredentials,
}

#[derive(Parser, Debug)]
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
    #[clap(long, value_enum, default_value_t = Grant::AuthorizationCodeWithPKCE, env = "DOKEN_GRANT")]
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

    /// OAuth 2.0 Client Identifier https://www.rfc-editor.org/rfc/rfc6749#section-2.2
    #[clap(long, env = "DOKEN_CLIENT_ID")]
    pub client_id: String,

    /// Port for callback url
    #[clap(long, default_value_t = 8081, env = "DOKEN_PORT")]
    pub port: u16,

    /// OAuth 2.0 Client Secret. Please use `--client-secret-stdin`, because it's not get stored in a shell history.  https://www.rfc-editor.org/rfc/rfc6749#section-2.3.1
    #[clap(long, env = "DOKEN_CLIENT_SECRET")]
    pub client_secret: Option<String>,

    /// OAuth 2.0 Client Secret from standard input https://www.rfc-editor.org/rfc/rfc6749#section-2.3.1
    #[clap(long, action, default_value_t = false)]
    pub client_secret_stdin: bool,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's username https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2
    #[clap(short, long, env = "DOKEN_USERNAME")]
    pub username: Option<String>,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's password https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2
    #[clap(short, long, env = "DOKEN_PASSWORD")]
    pub password: Option<String>,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's password from standard input https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2
    #[clap(long, action, default_value_t = false)]
    pub password_stdin: bool,

    /// OAuth 2.0 Scope https://www.rfc-editor.org/rfc/rfc6749#section-3.3
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
    }

    fn assert_grant_specific_arguments(args: &Arguments) {
        let mut cmd: Command = Arguments::command();

        match args.grant {
            Grant::AuthorizationCodeWithPKCE { .. } => {
                Self::assert_urls_for_authorization_grants(args);
            }
            Grant::AuthorizationCode { .. } => {
                Self::assert_urls_for_authorization_grants(args);
            }
            Grant::ResourceOwnerPasswordClientCredentials { .. } => {
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
            Grant::ClientCredentials { .. } => {
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
            Grant::Implicit { .. } => {
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

    pub fn parse() -> Arguments {
        log::debug!("Parsing application arguments...");
        if dotenv().is_ok() {
            log::debug!(".env file found");
        } else {
            log::debug!(".env file not found. skipping...");
        }

        let args = Arguments::parse();
        Self::assert_grant_specific_arguments(&args);
        let mut args = Self::parse_client_secret(args);
        args = Self::parse_password(args);

        log::debug!("Argument parsing done");
        log::debug!("Running with arguments: {:#?}", args);

        args
    }
}
