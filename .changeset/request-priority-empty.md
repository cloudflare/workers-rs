---
"workers-rs": patch
---

`Cf::request_priority` returns `None` instead of panicking when `requestPriority` is an empty string or a segment without `key=value`.
