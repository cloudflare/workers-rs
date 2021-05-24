# workers-rs

Abandon all hope, ye who enter here.

Or don't, I guess. Do your thing.

## repo layout

- edgeworker-sys = same as web-sys, and even some copy/pasted externs. these need to be slimmed down to only be what worker runtime supports, and added to with stuff I haven't got to
- macros = cf macro is the one that hoists your code into a "glue" conversion thing.. not super important, but makes it automatically nicer to work with
- worker = the convenience wrapper types & fn's on top of edgeworker-sys
- rust-sandbox = the example worker I use to play with all this stuff
