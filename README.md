# doken
[![Rust](https://github.com/RiddleMan/doken/actions/workflows/rust.yml/badge.svg)](https://github.com/RiddleMan/doken/actions/workflows/rust.yml)

Tool for getting tokens from OAuth 2.0/OpenID Connect providers.

## Features
* Retrieving token using _Authorization Code_, _Authorization Code with PKCE_, _Resource Owner Password Client Credentials_, _Client Credentials_, _Implicit_ grants
* Refreshing token without opening a browser if IdP provides _refresh_token_
* Reading options from CLI Arguments, Environment variables, _.env_ file

## Installation

```shell
brew tap RiddleMan/tap && brew install doken
```

## Usage

### ⚠️ _Authorization code_ and _Authorization Code with PKCE_ notice

If you want to use these grants firstly you have to set up your identity provider to allow http://localhost:8081/ callback url. If you want to change a port you could use `--port` argument for these grants.

### Basic _Authorization Code with PKCE_ grant

The most common use case that most of the Identity Providers support (if your IdP doesn't support PKCE, please use _authorization-code_ grant). 

It opens a browser to pass the grant and authorize. Returns an _access_token_ as an output.

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --client-id <client_id>
```

### Providing arguments as environment variables

If you have an Identity Provider you constantly request, then you provide all of them using environment variables. Every argument could be passed as the following `DOKEN_MY_ARGUMENT` ex. `DOKEN_TOKEN_URL`.

```shell
export DOKEN_TOKEN_URL=https://my-idp.com/oauth/token 
export DOKEN_AUTHORIZATION_URL=https://my-idp.com/authorize
export DOKEN_CLIENT_ID=<client_id>
export DOKEN_SECRET_ID=<client_secret>
export DOKEN_PORT=8081
export DOKEN_GRANT=authorization-code

doken
```

### Sharing project's IdP settings via `.env` file

The tool detects the closest `.env` file in a tree structure to make it easier to execute repeatable commands within the scope of the project. To do so create a _.env_ file at the root of your project with the following contents:

```
DOKEN_TOKEN_URL=https://my-idp.com/oauth/token 
DOKEN_AUTHORIZATION_URL=https://my-idp.com/authorize
DOKEN_CLIENT_ID=<client_id>
DOKEN_SECRET_ID=<client_secret>
DOKEN_PORT=8081
DOKEN_GRANT=authorization-code
```

Then retrieving a token goes like this:

```shell
doken
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
  --client-id <client_id> \
  --client-secret-stdin
```
### _Authorization Code with PKCE_ grant with custom port

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --client-id <client_id> \
  --port 8081
```

### _Client credentials_ grant with discovery url

```shell
doken \
  --discovery-url https://my-idp.com/.well-known/openid-configuration \
  --client-id <client_id> \
  --client-secret-stdin \
  --grant client-credentials
```

### _Implicit_ grant

⚠️ Not recommended. Use [Authorization Code with PKCE Grant](#basic-authorization-code-with-pkce-grant) instead. Read more: [link](https://auth0.com/docs/get-started/authentication-and-authorization-flow/implicit-flow-with-form-post#how-it-works).

```shell
doken \
  --discovery-url https://my-idp.com/.well-known/openid-configuration \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
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

## Token refresh details

The command tries to open a browser as rarely as possible. To achieve that the state (`~/.doken.json`) and refresh logic has been implemented.

Running the command in any of the authorization grants could result in one of these situations:

1. If no data about _client_id_ in `~/.doken.json`, then open a browser get token, save it in the state and output to the user
2. If _access_token_ is available in the state, and it's valid, then output to the user
3. If _access_token_ is invalid and _refresh_token_ exists and it's valid, then refresh token, save in the state and output to the user
4. If _access_token_ and _refresh_token_ are invalid, then remove state and use case no. 1

## License
`doken` is under the terms of the MIT License.

See the [LICENSE](LICENSE) file for details.
