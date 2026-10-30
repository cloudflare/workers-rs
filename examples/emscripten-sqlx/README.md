# Emscripten SQLx example

> **Experimental Preview**

```sh
git clone https://github.com/cloudflare/workers-rs
cd workers-rs/examples/emscripten-sqlx
npx wrangler dev
```

Connects to PostgreSQL through a Hyperdrive binding using SQLx, runs SELECT uuidv7()::text and returns the generated UUID.

This requires  PostgreSQL database running. Update wranger.toml with the correct connection string if needed.
