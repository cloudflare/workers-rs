---
"workers-rs": minor
---

Add `durable_object`-scheduling-policy `Container::start` options
(`image`, `instance`, `container_snapshot`, `labels`) and a
`Container::exec`/`ExecProcess`/`ExecOutput` binding for running and reading
the exit status of a process inside an already-running container.

`ContainerStartupOptions` previously exposed only `entrypoint`,
`enable_internet`, and `env`, which are the fields the `default` scheduling
policy's `start()` call accepts. The `durable_object` scheduling policy's
`ctx.container.start()` additionally accepts `image` (required unless
`containerSnapshot` is set), `instance` (named size or custom
`{ vcpu, memoryMib, diskMb}`), `containerSnapshot`, and `labels`. These are
now available via `ContainerStartupOptions::set_image`,
`set_instance`/`set_custom_instance`, `set_container_snapshot`, and
`add_label`.

`Container` previously had no binding for `exec()`, so a Rust Worker could
start a container and `wait_for_exit()` but could not run an additional
process inside it or read a granular exit code without destroying the whole
container. `Container::exec(cmd, options)` now returns an `ExecProcess` with
`pid()`, `exit_code()`, `output()` (an `ExecOutput` with `exit_code()`,
`stdout()`, `stderr()`), and `kill()`.

Source: <https://developers.cloudflare.com/containers/api/durable-object-container/>
and <https://developers.cloudflare.com/containers/configuration/scheduling-policy/>,
checked 2026-10-02.
