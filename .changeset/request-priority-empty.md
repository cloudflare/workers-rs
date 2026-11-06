---
"workers-rs": patch
---

`Cf::request_priority` returns `None` instead of panicking when `requestPriority` is empty, malformed or has no `weight`, and ignores unknown keys.
