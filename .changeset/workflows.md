---
"workers-rs": minor
---

Add [Workflows](https://developers.cloudflare.com/workflows/) support behind the `workflow` feature.

To define a workflow, put `#[workflow]` on a struct and implement `WorkflowEntrypoint` for it with typed `Input` and `Output`. Steps run through `WorkflowStep::do_`, which has variants that take a retry config or a saga-style rollback handler, and `do_stream` for steps that return a `ReadableStream`. `WorkflowStep` also has `sleep`, `sleep_until`, and `wait_for_event`. Returning a `NonRetryableError` fails the workflow without retrying.

`Env::workflow` returns the binding. `Workflow` and `WorkflowInstance` create instances, check their status, pause, resume, restart, terminate, or delete them, and send them events.

`worker-build` exports each `#[workflow]` struct from `build/index.js` as a subclass of `WorkflowEntrypoint`, so the `class_name` in `wrangler.toml` is just the struct's name.
