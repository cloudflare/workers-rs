# AbortSignal example

Cancel in-flight fetch requests using `AbortController` and `AbortSignal` in a Rust Cloudflare Worker.

## Routes

| Route | Description |
|---|---|
| `GET /abort?url=<url>` | Fetches the URL and immediately aborts. Always returns an abort error. |
| `GET /timeout?url=<url>&timeout=<ms>` | Fetches the URL with a timeout (default 2000ms). Cancels the request if the server doesn't respond in time. |

## Local slow server

A slow Node server is included for testing timeouts locally:

```sh
node slow-server.mjs              # port 3000, 5s delay
node slow-server.mjs 9000 10      # port 9000, 10s delay
```

The server has two endpoints:
- `GET /` returns a response after the default delay
- `GET /delay/:ms` returns a response after `:ms` milliseconds

## Testing

1. Start the slow server on a port (e.g. 3000):
   ```sh
   node slow-server.mjs 3000
   ```

2. Start the Worker (in the `abort-signal` example directory):
   ```sh
   npx wrangler dev
   ```

3. Test immediate abort:
   ```sh
   curl "http://localhost:8787/abort?url=http://localhost:3000"
   # → "Aborted: ..."
   ```

4. Test timeout (500ms timeout against a 5s delayed server):
   ```sh
   curl "http://localhost:8787/timeout?url=http://localhost:3000&timeout=500"
   # → "Request timed out after 500ms"
   ```

5. Test timeout where the server responds in time (using `/delay/100` for 100ms):
   ```sh
   curl "http://localhost:8787/timeout?url=http://localhost:3000/delay/100&timeout=2000"
   # → "Got response: {\"delayed_ms\":100,\"message\":\"slow response\"}"
   ```
