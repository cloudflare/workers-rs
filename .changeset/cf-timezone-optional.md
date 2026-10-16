---
"workers-rs": patch
---

Make `Cf::timezone_name()` return `Option<String>` instead of panicking when the request has no `cf.timezone`, following the earlier change to `as_organization` and `asn`.
