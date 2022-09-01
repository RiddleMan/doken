# doken
[![Rust](https://github.com/RiddleMan/doken/actions/workflows/rust.yml/badge.svg)](https://github.com/RiddleMan/doken/actions/workflows/rust.yml)

Tool for getting tokens from OAuth 2.0/OpenID Connect providers.

## Features
* Retrieving token using _Authorization Code_, _Authorization Code with PKCE_ flows
* Refreshing token without opening a browser if IdP provides _refresh_token_
* Reading options from CLI Arguments, Environment variables, _.env_ file

## Usage

### ⚠️ _Authorization code_ and _Authorization Code with PKCE_ notice

If you want to use these flows firstly you have to set up your identity provider to allow http://localhost:8081/ callback url. If you want to change a port you could use `--port` argument for these flows.

### Basic _Authorization Code with PKCE_ flow

The most common use case that most of the Identity Providers support (if your IdP doesn't support PKCE, please use _authorization-code_ flow). 

It opens a browser to pass the flow and authorize. Returns an _access_token_ as an output.

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --client-id <client_id> \
  authorization-code-with-pkce
```

### _Authorization Code with PKCE_ flow with secret

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --client-id <client_id> \
  --client-secret <client_secret> \
  authorization-code-with-pkce
```
### _Authorization Code with PKCE_ flow with custom port

```shell
doken \
  --token-url https://my-idp.com/oauth/token \
  --authorization-url https://my-idp.com/authorize \
  --client-id <client_id> \
  --client-secret <client_secret> \
  authorization-code-with-pkce \
  --port 8081
```

### Providing arguments as environment variables

If you have an Identity Provider you constantly request, then you provide all of them using environment variables. Every argument could be passed as the following `DOKEN_MY_ARGUMENT` ex. `DOKEN_TOKEN_URL`.

```shell
export DOKEN_TOKEN_URL=https://my-idp.com/oauth/token 
export DOKEN_AUTHORIZATION_URL=https://my-idp.com/authorize
export DOKEN_CLIENT_ID=<client_id>
export DOKEN_SECRET_ID=<client_secret>
export DOKEN_PORT=8081

doken authorization-code-with-pkce
```

### Sharing project's IdP settings via `.env` file

The tool detects the closest `.env` file in a tree structure to make it easier to execute repeatable commands within the scope of the project. To do so create a _.env_ file at the root of your project with the following contents:

```
DOKEN_TOKEN_URL=https://my-idp.com/oauth/token 
DOKEN_AUTHORIZATION_URL=https://my-idp.com/authorize
DOKEN_CLIENT_ID=<client_id>
DOKEN_SECRET_ID=<client_secret>
DOKEN_PORT=8081
```

Then retrieving a token goes like this:

```shell
doken authorization-code-with-pkce
```

### Usage with cURL

The power of this tool is the best while used with any request tools like _cURL_. Here's an example:

```shell
curl -H "Authorization: Bearer $(doken authorization-code-with-pkce)" https://my-api-url.com/users
```

## Token refresh details

The command tries to open a browser as rarely as possible. To achieve that the state (`~/.doken.json`) and refresh logic has been implemented.

Running the command in any of the authorization flows could result in one of these situations:

1. If no data about _client_id_ in `~/.doken.json`, then open a browser get token, save it in the state and output to the user
2. If _access_token_ is available in the state, and it's valid, then output to the user
3. If _access_token_ is invalid and _refresh_token_ exists and it's valid, then refresh token, save in the state and output to the user
4. If _access_token_ and _refresh_token_ are invalid, then remove state and use case no. 1

## License
`doken` is under the terms of the MIT License.

See the [LICENSE](LICENSE) file for details.
