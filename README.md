# Lift Proxy

A proxy server that handles rotation of socks5 proxies and auth tokens for LLM providers.

## Usage

There is currently no UI for managing auth tokens,
auth tokens can be added to the database manually.

### Routes

- GET `/`: Health check
- POST `/auths`: Update auth tokens to and from the database
- PUT `/auths`: Drop all auth tokens in memory and refetch from the database
- GET `/{proxy_flag}/{provider_name}/v1/models`: List models
- POST `/{proxy_flag}/{provider_name}/v1/chat/completions`: Chat completions
  - `proxy_flag`: `x` no proxy; `o` proxy on
  - `provider_name`: The provider name, defined by macro `impl_provider!()` in `src/providers/mod.rs`

## Getting Started

### Prerequisites

- Docker or Postgres
- Nix
  - devenv
  - direnv [optional]
- Without Nix
  - rust
  - cargo-shuttle
  - openssl

### Enter Dev Shell

Allow `direnv` to manage the environment

```sh
direnv allow
```

or enter the dev shell manually

```sh
devenv shell
```

### Local Run

```sh
shuttle run
```

### Deploy to [Shuttle](https://www.shuttle.dev/)

```sh
shuttle deploy
```

### Secrets

- `WEBSHARE_TOKEN`: WebShare API token for fetching proxies
- `AUTH_SECRET`: Bearer token for API calling authentication
