# TCP Socket Example

This example demonstrates handling inbound TCP connections in a Cloudflare Worker using the `#[event(connect)]` handler.

## Usage

Build and deploy the Worker:

```bash
npm install
npm run dev
```

## Handler

The `connect` handler receives a [`Socket`](https://docs.rs/worker/latest/worker/struct.Socket.html) for each inbound TCP connection and can read/write data using the standard `tokio::io` traits.
