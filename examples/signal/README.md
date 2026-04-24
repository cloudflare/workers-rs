# CPU Limit Handling

Rust Workers can respond to Worker CPU limits by listening for CPU limit approaching signal.

This is available via the `worker::signals` API.

If the signal is not respected and CPU work backed off from for the current execution, then the Rust Worker will be terminated.

This example demonstrates how to listen for these signals.

_Note: This signal is currently only supported on the Edgeworker CDN runtime, not on local Miniflare instances._

### Example

To run the example, run `npx wrangler deploy` from this repo. As many digits of PI will be computed as is supported
by the platform before running out of CPU time for the worker.

The example here computes digits of PI, a computationally expensive operation, by wrapping the hot loop with
checks to `worker::signals::is_near_cpu_limit()`:

```rs
    while !signal::is_near_cpu_limit() {
        if &q * &four + &r - &t < &n * &t {
            // The extraction check passed — `n` is the next confirmed digit of Pi.
            result.push_str(&n.to_string());
            if first_digit {
                result.push('.');
                first_digit = false;
            }

            // Remap the state to shift out the extracted digit.
            let new_r = (&r - &n * &t) * &ten;
            let new_n = ((&q * &three + &r) * &ten).div_floor(&t) - &n * &ten;
            q *= &ten;
            r = new_r;
            n = new_n;
        } else {
            // Absorb the next term of the continued fraction.
            let new_r = (&q * &two + &r) * &l;
            let tl = &t * &l;
            let new_n = (&q * (&k * &seven + &two) + &r * &l).div_floor(&tl);
            q *= &k;
            t = tl;
            l += &two;
            k += &one;
            r = new_r;
            n = new_n;
        }
    }
```
