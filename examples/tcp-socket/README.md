# TCP Socket Example

This example demonstrates handling inbound TCP connections in a Cloudflare Worker using the `#[event(connect)]` handler.

## Usage

Build and deploy the Worker:

```bash
npx worker-build
npx wrangler deploy
```

## Handler

The `connect` handler receives a [`Socket`](worker::Socket) for each inbound TCP connection and can read/write data using the standard `tokio::io` traits.
