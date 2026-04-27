use num_bigint::BigInt;
use num_integer::Integer;
use worker::*;

#[event(fetch)]
async fn fetch(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let pi = compute_pi();
    // The digit count is the string length minus the "3." prefix.
    let digit_count = pi.len().saturating_sub(2);

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Pi Computation</title>
  <style>
    body {{ font-family: monospace; max-width: 48rem; margin: 2rem auto; padding: 0 1rem; }}
    h1 {{ margin-bottom: 0.25rem; }}
    .count {{ color: #555; margin-bottom: 1rem; }}
    .digits {{ word-break: break-all; white-space: pre-wrap; }}
  </style>
</head>
<body>
  <h1>Pi</h1>
  <p class="count">{digit_count} digits after the decimal point</p>
  <p class="digits">{pi}</p>
</body>
</html>"#
    );

    Response::from_html(html)
}

/// Compute decimal digits of Pi until the runtime signals that time is almost up.
///
/// Uses the unbounded spigot algorithm (Gibbons, 2006) which streams digits of Pi
/// from a generalized continued-fraction expansion. All arithmetic is performed
/// with arbitrary-precision integers — no floating-point is involved.
///
/// The signal is checked once per loop iteration. After it fires, the only work
/// done is a branch and a return of the already-built String — O(1).
fn compute_pi() -> String {
    let mut result = String::with_capacity(8192);

    // Precompute small BigInt constants to avoid type-inference ambiguity
    // and repeated conversions in the hot loop.
    let one = BigInt::from(1u32);
    let two = BigInt::from(2u32);
    let three = BigInt::from(3u32);
    let four = BigInt::from(4u32);
    let seven = BigInt::from(7u32);
    let ten = BigInt::from(10u32);

    let mut q = BigInt::from(1u32);
    let mut r = BigInt::from(0u32);
    let mut t = BigInt::from(1u32);
    let mut k = BigInt::from(1u32);
    let mut n = BigInt::from(3u32);
    let mut l = BigInt::from(3u32);
    let mut first_digit = true;

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

    result
}
