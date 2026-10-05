# Workflows on Cloudflare Workers

Durable, multi-step [Cloudflare Workflows](https://developers.cloudflare.com/workflows/) written in Rust with the `#[workflow]` macro and `worker::WorkflowEntrypoint`.

`MyWorkflow` (in `src/lib.rs`) runs a small pipeline and exercises most of the Workflows surface:

- **Durable steps.** `step.do_(...)` and `step.do_with_config(...)` persist their return values and replay them instead of re-running.
- **Retries & backoff.** `StepConfig` / `RetryConfig` set a per-step `limit`, `delay`, and `Backoff` strategy. The `send-notification` step fails randomly to show retries in action.
- **Non-retryable errors.** `NonRetryableError` fails the instance immediately (used when the email is invalid).
- **Sleeping.** `step.sleep(...)` hibernates the instance (10 seconds here) without burning CPU.
- **Saga rollback.** `step.do_with_rollback(...)` attaches a compensation handler that the runtime invokes if a later step fails. See `reserve-inventory`.
- **Cron triggers.** When started from a `schedules` cron (see `wrangler.toml`), `event.schedule` and `event.workflow_name` are populated.

A `fetch` handler is included so you can drive the workflow over HTTP.

## Routes

| Route | Description |
|---|---|
| `POST /workflow` | Create an instance. Body: `{ "email": "...", "name": "..." }`. Returns the instance `id`. |
| `GET /workflow/{id}` | Get an instance's status, output, and error. |
| `POST /workflow/{id}/pause` | Pause a running instance. |
| `POST /workflow/{id}/resume` | Resume a paused instance. |

## Running locally

```sh
npx wrangler dev
```

`wrangler dev` runs the `[build]` command from `wrangler.toml`, which invokes `worker-build`. If you don't have it, install it first with `cargo install worker-build` (and point the build command at it, or drop the `../../target/release/` prefix).

> Saga rollbacks are a recent Workflows feature. Local development needs a current `wrangler`/`workerd`; an older toolchain will run the steps but skip the rollback.

### Walkthrough

1. Create an instance:

   ```sh
   curl -X POST http://localhost:8787/workflow \
     -H 'Content-Type: application/json' \
     -d '{"email":"user@example.com","name":"Ada"}'
   # → {"id":"<instance-id>","message":"Workflow created"}
   ```

2. Poll its status (it sleeps for 10s mid-run, so give it a moment):

   ```sh
   curl http://localhost:8787/workflow/<instance-id>
   # → {"id":"...","status":"Running","error":null,"output":null}
   # ...later...
   # → {"id":"...","status":"Complete","output":{"message":"Workflow completed for Ada","steps_completed":4}}
   ```

   Passing an email without an `@` makes the `validate-params` step throw a
   `NonRetryableError`, so the instance ends up `Errored` instead.

3. Pause / resume while it's running (e.g. during the 10s sleep):

   ```sh
   curl -X POST http://localhost:8787/workflow/<instance-id>/pause
   curl -X POST http://localhost:8787/workflow/<instance-id>/resume
   ```

## Deploying

```sh
npx wrangler deploy
```
