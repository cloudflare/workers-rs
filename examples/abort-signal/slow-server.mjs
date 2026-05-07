// A minimal HTTP server that responds after a configurable delay.
// Used to test AbortSignal timeouts against a real slow endpoint.
//
// Usage:
//   node slow-server.mjs            # default 5s delay on port 3000
//   node slow-server.mjs 9000 10    # port 9000, 10s delay
//
// Endpoints:
//   GET /           → responds after <delay> seconds
//   GET /delay/:ms  → responds after :ms milliseconds

import { createServer } from "node:http";

const PORT = parseInt(process.argv[2] || "3000", 10);
const DEFAULT_DELAY_S = parseInt(process.argv[3] || "5", 10);

const server = createServer((req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  const match = url.pathname.match(/^\/delay\/(\d+)$/);
  const delayMs = match
    ? parseInt(match[1], 10)
    : DEFAULT_DELAY_S * 1000;

  console.log(`${req.method} ${url.pathname} → will respond in ${delayMs}ms`);

  const timer = setTimeout(() => {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ delayed_ms: delayMs, message: "slow response" }));
  }, delayMs);

  req.on("close", () => clearTimeout(timer));
});

server.listen(PORT, () => {
  console.log(`Slow server listening on http://localhost:${PORT}`);
  console.log(`Default delay: ${DEFAULT_DELAY_S}s`);
});
