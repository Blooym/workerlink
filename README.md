# Workerlink

A fully serverless URL shortener built on Cloudflare Workers & Cloudflare KV in under ~500 lines of Rust.

**Project status: Completed & Maintained**

## Deployment

In order to deploy Workerlink to Cloudflare Workers, you need to do the following:

1. Clone this repository locally with git or by downloading the source archive.
2. Download and install [NodeJS](https://nodejs.org) and [Rust v1.75.0+](https://rustup.rs/), or use the provided [.devcontainer](.devcontainer) setup for an environment.
3. Run `npm install` to install all dependencies needed to build/deploy.
4. [Setup a KV namespace](https://developers.cloudflare.com/kv/get-started/) on Cloudflare by following their documentation.
5. Create a 'wrangler.toml' file with the following contents at the root of the repository:
    ```toml
    name = "workerlink"
    main = "build/worker/shim.mjs"
    compatibility_date = "2023-12-01"
    kv_namespaces = [
        { binding = "links", id = "<KV ID>" } # Replace <KV ID> with the ID of the KV namespace you setup earlier.
    ]

    [vars]
    AUTH_TOKEN = "" # Set this to the token you want to use for authentication.

    [build]
    command = "cargo install -q worker-build && worker-build --release"
    ```
6. Run `npm run deploy` to deploy the worker to Cloudflare; You will be prompted to authenticate with Cloudflare during this process so the worker can be deployed using your account.

## Examples

- **In a browser:** Use a redirect.  
Navigate to `https://<WORKER_URL>/<ID>` and the browser will automatically direct.

- **In a browser:** See where an ID redirects to.  
Navigate to `https://<WORKER_URL>/<ID>/where` and the redirect url will be shown in plaintext.

- **Using curl:** Create/Update a new redirect.
    ```bash
    curl --request POST \
      --url 'https://<WORKER_URL>/<ID>' \
      --header 'Authorization: <AUTH_TOKEN>' \
      --header 'content-type: application/json' \
      --data '{
      "url": "<URL_TO_REDIRECT_TO>",
      "expiry_timestamp": unix_timestamp | null,
      "max_views": number | null,
      "overwrite": boolean,
      "disabled": boolean
    }'
    ```

- **Using curl:** Delete an existing redirect.
    ```bash
    curl --request DELETE \
      --url 'https://<WORKER_URL>/<ID>' \
      --header 'Authorization: <AUTH_TOKEN>'
    ```

- **Using curl:** Check the underlying JSON of a redirect.
    ```bash
    curl 'https://<WORKER_URL>/<ID>/details' \
        --header 'Authorization: <AUTH_TOKEN>'
    ```

## Licence

This project is dual-licenced under both the MIT Licence and the Apache Licence (Version 2.0). See [LICENCE-MIT](LICENCE-MIT) and [LICENCE-APACHE](LICENCE-APACHE) for more details.
