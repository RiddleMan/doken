use clap::{ArgEnum, ArgGroup, Command, CommandFactory, ErrorKind, Parser};
use dotenv::dotenv;
use std::error::Error;
use std::io;

#[derive(ArgEnum, Clone, Debug)]
pub enum Flow {
    /// Authorization code with PKCE flow. More: https://www.rfc-editor.org/rfc/rfc7636
    AuthorizationCodeWithPKCE,
    /// Authorization code flow. More: https://www.rfc-editor.org/rfc/rfc6749#section-1.3.1
    AuthorizationCode,
    // TODO: Implement flows
    // /// Implicit flow. More: https://www.rfc-editor.org/rfc/rfc6749#section-1.3.2
    // Implicit,
    /// Client credentials flow. More: https://www.rfc-editor.org/rfc/rfc6749#section-1.3.4
    ClientCredentials,
}

#[derive(ArgEnum, Clone, Debug)]
pub enum TokenType {
    IdToken,
    AccessToken,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(group(
    ArgGroup::new("oauth2")
        .multiple(true)
        .args(&["token-url", "authorization-url"])
        .conflicts_with("oidc")
))]
#[clap(group(
    ArgGroup::new("oidc")
        .arg("discovery-url")
        .conflicts_with("oauth2")
))]
pub struct Arguments {
    /// Authentication flow
    #[clap(long, arg_enum, default_value_t = Flow::AuthorizationCodeWithPKCE, env = "DOKEN_FLOW")]
    pub flow: Flow,

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

    /// OAuth 2.0 Scope https://www.rfc-editor.org/rfc/rfc6749#section-3.3
    #[clap(long, default_value = "offline_access", env = "DOKEN_SCOPE")]
    pub scope: String,

    /// OpenID Connect requested aud
    #[clap(long, env = "DOKEN_AUDIENCE")]
    pub audience: Option<String>,

    /// When turned on ignores the state file and continues with a fresh flow
    #[clap(short, long, action, default_value_t = false)]
    pub force: bool,

    /// Add diagnostics info
    #[clap(short, long, action, default_value_t = false)]
    pub debug: bool,

    /// Token type: OpenID Connect ID Token or OAuth 2.0 Access Token
    #[clap(long, arg_enum, default_value_t = TokenType::AccessToken, env = "DOKEN_TOKEN_TYPE")]
    pub token_type: TokenType,
}

pub struct Args;

// TODO: match green color as the rest of clap messages
impl Args {
    fn assert_urls_for_authorization_flows(args: &Arguments) {
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

    fn assert_flow_specific_arguments(args: &Arguments) {
        let mut cmd: Command = Arguments::command();

        match args.flow {
            Flow::AuthorizationCodeWithPKCE { .. } => {
                Self::assert_urls_for_authorization_flows(args);
            }
            Flow::AuthorizationCode { .. } => {
                Self::assert_urls_for_authorization_flows(args);
            }
            Flow::ClientCredentials { .. } => {
                if args.authorization_url.is_some() {
                    cmd.error(
                        ErrorKind::ArgumentConflict,
                        "--authorization-url cannot be used with:\n\t--flow client-credentials",
                    )
                    .exit();
                }

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
                        "--client-secret or --client-secret-stdin is required while used with `client-credentials` flow.",
                    )
                        .exit();
                }
            }
        }
    }

    fn parse_client_secret(mut args: Arguments) -> Result<Arguments, Box<dyn Error>> {
        if args.client_secret.is_some() && std::env::var("DOKEN_CLIENT_SECRET").is_err() {
            eprintln!("Please use `--client-secret-stdin` as a more secure variant.");
        }

        if args.client_secret_stdin {
            let mut client_secret = String::new();
            eprint!("Client Secret: ");
            io::stdin().read_line(&mut client_secret)?;
            args.client_secret = Some(client_secret.trim().to_string());
        }

        Ok(args)
    }

    pub fn parse() -> Result<Arguments, Box<dyn Error>> {
        log::debug!("Parsing application arguments...");
        if dotenv().is_ok() {
            log::debug!(".env file found");
        } else {
            log::debug!(".env file not found. skipping...");
        }

        let args = Arguments::parse();
        Self::assert_flow_specific_arguments(&args);
        let args = Self::parse_client_secret(args)?;

        log::debug!("Argument parsing done");
        log::debug!("Running with arguments: {:#?}", args);

        Ok(args)
    }
}
