# Receiving email on Cloudflare Workers

Example of an [Email Worker](https://developers.cloudflare.com/email-routing/email-workers/) that auto-replies with `message received` to every inbound message.

The handler is wired up via `#[event(email)]`. It receives a [`worker::InboundEmail`](https://docs.rs/worker/latest/worker/struct.InboundEmail.html), pulls `Message-ID` and `Subject` off the original headers, assembles a reply with [`mail-builder`](https://crates.io/crates/mail-builder), and hands it to [`InboundEmail::reply`](https://docs.rs/worker/latest/worker/struct.InboundEmail.html#method.reply).

Unlike `[[send_email]]`, inbound delivery is not declared in `wrangler.toml` — once deployed, attach an address to this worker from the **Email → Email Routing** section of the Cloudflare dashboard.

## Local development

`wrangler dev --local` exposes a `/cdn-cgi/handler/email` endpoint that simulates an inbound delivery. The reply is written to a local `.eml` file rather than actually being sent (see the [Cloudflare docs](https://developers.cloudflare.com/email-routing/email-workers/local-development/)).

```bash
npm install
npm run dev
# then, in another shell:
curl -X POST 'http://localhost:8787/cdn-cgi/handler/email' \
  --url-query 'from=sender@example.com' \
  --url-query 'to=recipient@example.com' \
  --header 'Content-Type: application/octet-stream' \
  --data-binary @sample.eml
```

`sample.eml` is any RFC 5322 message, e.g.:

```
From: sender@example.com
To: recipient@example.com
Subject: hello
Message-ID: <abc@example.com>

hi there
```

## Deploying

```bash
npm run deploy
```

After deploying, go to **Email → Email Routing → Email Workers** in the Cloudflare dashboard and route an address to `receive-email-on-workers`.

## Replies require valid DMARC on the sender's domain

Cloudflare's `reply()` only succeeds if the inbound message passes DMARC ([docs](https://developers.cloudflare.com/email-routing/email-workers/reply-email-workers/#requirements)). A *missing* DMARC record at `_dmarc.<sender-domain>` is treated as a failure (`policy.dmarc=none`) and the runtime throws `original email cannot be replied to`. SPF and DKIM passing on their own are not enough.

If you're testing from a domain you control and the reply fails, publish a minimal record — monitor-only `p=none` is sufficient:

```
_dmarc.example.com.  TXT  "v=DMARC1; p=none;"
```
