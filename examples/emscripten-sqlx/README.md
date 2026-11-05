# Emscripten SQLx example

> **Experimental Preview**

```sh
git clone https://github.com/cloudflare/workers-rs
cd workers-rs/examples/emscripten-sqlx
npx wrangler dev
```

Connects to PostgreSQL through a Hyperdrive binding using SQLx, runs `SELECT uuidv7()::text` and returns the generated UUID.

This requires a running PostgreSQL 18+ database (for `uuidv7()`). Update `localConnectionString` in `wrangler.toml` if needed.
