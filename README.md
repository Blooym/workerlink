# Workerlink

A fully serverless URL shortener built on Cloudflare Workers & Cloudflare KV.

## Deployment

In order to deploy Workerlink to Cloudflare Workers, you need to do the following:

1. Clone this repository locally with git or by downloading the source archive.
2. Download and install the latest [NodeJS LTS](https://nodejs.org) and [Rust](https://rustup.rs/) versions, or use the provided [.devcontainer](.devcontainer) setup for an environment.
3. Run `npm install` to install all dependencies needed to build/deploy.
4. [Setup a KV namespace](https://developers.cloudflare.com/kv/get-started/) on Cloudflare by following their documentation.
5. Create a 'wrangler.toml' file with the following contents at the root of the repository:
    ```toml
    name = "workerlink"
    main = "build/worker/shim.mjs"
    compatibility_date = "2023-08-10"
    kv_namespaces = [
        { binding = "links", id = "<KV ID>" } # Replace <KV ID> with the ID of the KV namespace you setup earlier.
    ]

    [vars]
    AUTH_TOKEN = # Set this to the token you want to use for authentication.

    [build]
    command = "cargo install -q worker-build && worker-build --release"
    ```
6. Run `npm run deploy` to deploy the worker to Cloudflare; You will be prompted to authenticate with Cloudflare during this process so the worker can be deployed using your account.

## API Documentation

All interaction/management actions for Workerlink is done via HTTP requests. Below outlines the structure for the API:

- **`GET /:id`:** Redirects to the URL associated with the ID if it exists.
    - Authentication Required: No

- **`POST /:id`:** Create or update a shortlink.
    - Authentication Required: Yes

- **`DELETE /:id`:** - Deletes a shortlink.

## Licence

This project is licenced under both the MIT Licence and the Apache Licence (Version 2.0). See [LICENCE-MIT](LICENCE-MIT) and [LICENCE-APACHE](LICENCE-APACHE) for more details.
