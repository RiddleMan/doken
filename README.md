# doken
[![Rust](https://github.com/RiddleMan/doken/actions/workflows/rust.yml/badge.svg)](https://github.com/RiddleMan/doken/actions/workflows/rust.yml)

Tool for getting tokens from OAuth 2.0/OpenID Connect providers.

## Features
* Retrieving token using _Authorization Code_, _Authorization Code with PKCE_, _Resource Owner Password Client Credentials_, _Client Credentials_, _Implicit_ grants
* Refreshing token without opening a browser if IdP provides _refresh_token_
* Reading options from CLI Arguments, Environment variables, _.env_ file

## Prerequisites

- Chromium-based browser (Edge, Chromium, Chrome)

## Installation

### Install pre-built binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/RiddleMan/doken/releases/latest/download/doken-installer.sh | sh
```

### Install pre-built binaries via PowerShell script

Ensure you run this command in [Adminstrative shell](https://www.howtogeek.com/194041/how-to-open-the-command-prompt-as-administrator-in-windows-10/).

```sh
Set-ExecutionPolicy Bypass -Scope Process -Force; irm https://github.com/RiddleMan/doken/releases/latest/download/doken-installer.ps1 | iex
```

### Install pre-built binaries via Homebrew

```sh
brew install RiddleMan/homebrew-tap/doken
```

## Usage

### Basic _Authorization Code with PKCE_ grant

The most common use case that most of the Identity Providers support (if your IdP doesn't support PKCE, please use _authorization-code_ grant). 

It opens a browser to pass the grant and authorize. Returns an _access_token_ as an output.

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --callback-url https://my-app-domain.com/oauth2/callback \
  --client-id <client_id>
```

### Providing arguments as environment variables

If you have an Identity Provider you constantly request, then you provide all of them using environment variables. Every argument could be passed as the following `DOKEN_MY_ARGUMENT` ex. `DOKEN_TOKEN_URL`.

```shell
export DOKEN_TOKEN_URL=https://my-idp.com/oauth/token 
export DOKEN_AUTHORIZATION_URL=https://my-idp.com/authorize
export DOKEN_CALLBACK_URL=https://my-app-domain.com/oauth2/callback
export DOKEN_CLIENT_ID=<client_id>
export DOKEN_SECRET_ID=<client_secret>
export DOKEN_GRANT=authorization-code

doken
```

### Sharing project's IdP settings via `.env` file

The tool detects the closest `.env` file in a tree structure to make it easier to execute repeatable commands within the scope of the project. To do so create a _.env_ file at the root of your project with the following contents:

```
DOKEN_TOKEN_URL=https://my-idp.com/oauth/token 
DOKEN_AUTHORIZATION_URL=https://my-idp.com/authorize
DOKEN_CALLBACK_URL=https://my-app-domain.com/oauth2/callback
DOKEN_CLIENT_ID=<client_id>
DOKEN_SECRET_ID=<client_secret>
DOKEN_GRANT=authorization-code
```

Then retrieving a token goes like this:

```shell
doken
```

### Saving multiple IdP profiles in ~/.doken/config.toml file

The tool allows you to store multiple profiles in TOML file like so:

```toml
# File ~/.doken/config.toml

# Example of all possible values
[profile.first_profile]
client_id = "<client_id>"
client_secret = "<client_secret>"
audience = "https://my-app-domain.com/api"
scope = "email profile"
authorization_url = "https://my-idp.com/authorize"
token_url = "https://my-idp.com/oauth/token"
callback_url = "https://my-app-domain.com/oauth2/callback"
grant = "authorization-code-with-pkce"

# Example of partially filled profile
[profile.second_profile]
discovery_url = "https://my-idp.com/discovery"
callback_url = "https://my-app-domain.com/oauth2/callback"
grant = "authorization-code-with-pkce"
```

To authorize against first profile you type:


```shell
doken --profile first_profile
```

To use second profile with different client_id/client_secret pair you execute:

```shell
doken --profile second_profile --client-id <client_id> --client-secret-stdin
```

There's even option to overwrite some of settings defined in profile by providing an argument in command line:

```shell
doken --profile first_profile --client-id <different_client_id>
```

If some option is required, but wasn't provided the tool will error:

```shell
$ doken --profile second_profile

error: the following required arguments were not provided:
  --client-id <CLIENT_ID>

Usage: doken --client-id <CLIENT_ID> --profile <PROFILE> --grant <GRANT> --callback-url <CALLBACK_URL>

For more information, try '--help'.
```

### Usage with cURL

The power of this tool is the best while used with any request tools like _cURL_. Here's an example:

```shell
curl -H "Authorization: Bearer $(doken)" https://my-api-url.com/users
```
### _Authorization Code with PKCE_ grant with secret

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --callback-url https://my-app-domain.com/oauth2/callback \
  --client-id <client_id> \
  --client-secret-stdin
```

### _Client credentials_ grant with discovery url

```shell
doken \
  --discovery-url https://my-idp.com/.well-known/openid-configuration \
  --callback-url https://my-app-domain.com/oauth2/callback \
  --client-id <client_id> \
  --client-secret-stdin \
  --grant client-credentials
```

### _Implicit_ grant

⚠️ Not recommended. Use [Authorization Code with PKCE Grant](#basic-authorization-code-with-pkce-grant) instead. Read more: [link](https://auth0.com/docs/get-started/authentication-and-authorization-flow/implicit-flow-with-form-post#how-it-works).

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --callback-url https://my-app-domain.com/oauth2/callback \
  --client-id <client_id> \
  --client-secret-stdin \
  --grant implicit
```

### _Resource Owner Password Client credentials_ grant

⚠️ Not recommended. Use [Authorization Code Grant with PKCE](#basic-authorization-code-with-pkce-grant) instead. Read more: [link](https://auth0.com/docs/get-started/authentication-and-authorization-flow/resource-owner-password-flow).

```shell
doken \
  --discovery-url https://my-idp.com/.well-known/openid-configuration \
  --client-id <client_id> \
  --client-secret-stdin \
  --username <my_username> \
  --password-stdin \
  --grant resource-owner-password-client-credentials
```

## Arguments priority

Doken gathers arguments to the command from various sources. Here's the list of least prioritized to the most, meaning that the last one overwrites values of the previous ones.

1. _.env_ file
2. Environment variables ex. _DOKEN_CLIENT_ID=<client_id>_
3. Profiles from _~/.doken/config.toml_
4. Command arguments ex. _--client-id <client_id>_

## Token refresh details

The command tries to open a browser as rarely as possible. To achieve that the state (`~/.doken.json`) and refresh logic has been implemented.

Running the command in any of the authorization grants could result in one of these situations:

1. If no data about _client_id_ in `~/.doken.json`, then open a browser get token, save it in the state and output to the user
2. If _access_token_ is available in the state, and it's valid, then output to the user
3. If _access_token_ is invalid and _refresh_token_ exists and it's valid, then refresh token, save in the state and output to the user
4. If _access_token_ and _refresh_token_ are invalid, then remove state and use case no. 1

## Frequently asked questions

### Can't find a correct location of `config.toml`

Location varies between operating systems:

- Windows - _C:\Users\<your_username>\.doken\config.toml_
- Mac - _/Users/<your_username>/.doken/config.toml_
- Linux - _/home/<your_username>/.doken/config.toml_

## License
`doken` is under the terms of the MIT License.

See the [LICENSE](LICENSE) file for details.
