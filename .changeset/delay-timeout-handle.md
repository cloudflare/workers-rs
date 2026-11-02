---
"workers-rs": patch
---

Accept the Node.js `Timeout` object that `setTimeout` returns under `nodejs_compat` (default from compatibility date 2026-08-04) as the `Delay` timer handle, so `Delay` no longer panics in `worker-build --dev` builds.
