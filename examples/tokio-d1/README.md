# tokio and d1 on Cloudflare Workers

Demonstration of using `tokio` and `d1` on Cloudflare Workers.

## Setup
1. `npx wrangler d1 create dev-d1-rust`

Now you can add your database to the configuration toml by replacing
`<YOUR_DATABASE_ID>`


2. Insert some records
`npx wrangler d1 execute dev-d1-rust --file=./database/schema.sql`



3. `npm run deploy`
