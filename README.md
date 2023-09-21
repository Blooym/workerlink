# shortlink-cf

A fully serverless URL shortener built on Cloudflare Workers & Cloudflare KV.

## Setup

In order to use this, you will need to create a 'wrangler.toml' file with the following contents:

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

After that, you can run `npm install` to install the dependencies, and `npm run deploy` to deploy the worker to Cloudflare.

## Routes 

- `GET /:id` - Redirects to the URL associated with the shortlink.
- `HEAD /:id` - Checks if the shortlink exists.
- `🔒 POST /:id` - Creates a shortlink.
- `🔒 PUT /:id` - Updates a shortlink.
- `🔒 DELETE /:id` - Deletes a shortlink.

### Authentication

The 🔒 icon denotes a request that requires a valid `Authentication` header to be provided alongside the request. This will be the same as the `AUTH_TOKEN` you've set as an environment variable.


## License

This project is licensed under the BSD 3-Clause Licence. See [LICENCE](LICENCE) for more information.
