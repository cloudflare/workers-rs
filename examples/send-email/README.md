# Sending Email from Cloudflare Workers

Demonstration of using `worker::SendEmail` to dispatch an outbound message
through a `[[send_email]]` binding.

The MIME body is built with
[`mail-builder`](https://crates.io/crates/mail-builder), wrapped in an
[`EmailMessage`](https://docs.rs/worker/latest/worker/struct.EmailMessage.html),
and handed to `env.send_email("EMAIL")`.

## Local development

Running `wrangler dev --local` does **not** actually deliver the email. Per
the [Cloudflare docs](https://developers.cloudflare.com/email-routing/email-workers/local-development/),
outbound messages are simulated by writing each one to a local `.eml` file —
the path is printed in the terminal so you can inspect the raw message.

```bash
npm install
npm run dev
# then, in another shell:
curl http://localhost:8787/
```

## Deploying

Before deploying, verify the sender and recipient addresses as documented at
<https://developers.cloudflare.com/email-routing/email-workers/send-email-workers/>,
then:

```bash
npm run deploy
```
