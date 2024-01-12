# shortlink-cf

> [!WARNING]  
> Shortlink is not ready for production usage just yet! Its API structure and storage system is subject to breaking changes for the time being.


A fully serverless URL shortener built on Cloudflare Workers & Cloudflare KV.

## Setup

In order to setup Shortlink on Cloudflare, create a 'wrangler.toml' file with the following contents:

```toml
name = "shortlink"
main = "build/worker/shim.mjs"
compatibility_date = "2023-08-10"
kv_namespaces = [
    { binding = "locations", id = "<KV ID>" } # Replace <KV ID> with the ID of the KV namespace you want to use, you may need to create one first.
]

[vars]
AUTH_TOKEN = # Set this to the token you want to use for authentication.

[build]
command = "cargo install -q worker-build && worker-build --release"
```

After that, run `npm install` to install the dependencies, and `npm run deploy` to deploy the worker to Cloudflare.

## Uses

Using the provided API you could easily setup Shortlink with any bash script to keybind creating a new short URL, or even integrate it with the "Shortcuts" app on Apple devices to shorten links from the comfort of a URL share sheet.

## API Route Documentation

🔒 represents a route that requires a valid `Authorization` header to be provided, this will be the same as the `AUTH_TOKEN` you've set as an environment variable.

- **`GET /:id`: Redirect to the URL associated with the ID if it exists.**

-  **`🔒 POST /:id`: Create or update a shortlink.**

    * Example body:
        ```json5
        {
            "url": "https://example.com", // The URL to redirect to upon visiting the link.
            "overwrite": false|true // Whether or not to overwrite any existing shortlink with the same ID.
        }
        ```

- **`🔒 DELETE /:id` - Deletes a shortlink.**

## Licence

This project is licenced under both the MIT Licence and the Apache Licence (Version 2.0). See [LICENCE-MIT](LICENCE-MIT) and [LICENCE-APACHE](LICENCE-APACHE) for more details.
