# shortlink-cf-workers

A fully serverless URL shortener built on Cloudflare Workers & Cloudflare KV

## Setup

In order to use this, you will need to create a 'wrangler.toml' file with the following contents:

```toml
name = "shortlink-cf-workers"
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

## Usage

### Getting a shortlink

To redirect with a shortlink, send a GET request to `/:id`. This is what you would use in a browser.

### Checking if a shortlink exists

To check if a shortlink exists, send a HEAD request to `/:id`. This will return a 200 status code if the shortlink exists, and a 404 status code if it does not.

### Creating a shortlink

To create a shortlink, send a POST request `/:id` with the following JSON body:

```json
{
    "url": "https://example.com" # Replace this with the URL you want to redirect to.
}
```

### Updating

To update a shortlink, send a PUT request to `/:id` with the following JSON body:

```json
{
    "url": "https://example.com" # Replace this with the URL you want to redirect to.
}
```

### Deleting a shortlink

To delete a shortlink, send a DELETE request to `/:id`.

