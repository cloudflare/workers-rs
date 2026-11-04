---
"workers-rs": patch
---

`Cf::request_priority` returns `None` instead of panicking when `requestPriority` is empty or malformed, and ignores unknown keys.
