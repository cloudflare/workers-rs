---
"workers-rs": patch
---

Send Durable Object `GetAlarmOptions` / `SetAlarmOptions` to the runtime as camelCase (`allowConcurrency`, `allowUnconfirmed`) so `get_alarm_with_options`, `set_alarm_with_options` and `delete_alarm_with_options` no longer silently drop the options.
