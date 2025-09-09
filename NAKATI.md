# Standard Library
## `class Seq[T]` — Dynamic Arrays
## `class Map[K, V]` — Mappings
## `interface Sequential[T]` — interface for iterable / sequential types
- ```
let a: Map[i8, u32] = Map.new();
// Map[K, V] implements Sequential[(K, V)]
for (k, v) in a {
    // k is i8
    // v is u32
}```

## OpenTelemetry API
```
import Tracing from "@nakati/telemetry";

Tracing.withSpan((span) => {
    span.addContext("idk");
    span.addAttributes({
        "name": "idk something",
        "ae": "owkgdsf",
    });
});
```

## Numeric Types
- `i*`: signed integer of a certain number of bits
- `u*`: unsigned integer of a certain number of bits
- `f*`: floating point number of a certain number of bits
- `Integer`: a dynamically sized integer value (like `int` from Python, `Integer` from Haskell, `BigInt` from JavaScript)
- `Integer[Range]`: an integer within a specific range
    - alias: `min..max` (e.g. `0..23` or `1900..2999`)
- `Rational[Precision]`: dynamically sized rational value (like `BigDecimal` from other languages). Precision is used for rounding the results of division.
- `Fraction`: dynamically sized fraction (like `Fraction` from other languages). Only proposed since Fractions can store a quotient that never terminates ("repeating decimals").
- `Rational[Range, Precision]`: a fixed point rational value with a certain amount of precision and within a specific range.
    - alias: `min..max@precision` (e.g. `(32.5)..(73.1)@0.1`)
    - Precision can be either an integer value (where it is treated as a power of two to use for the precision. `2` = `1/4`s, `3` = `1/8`s) or a rational literal (where the number of decimal places it has it used to determine the precision).
    - future thought: could this be used to say that you only need the tens place level of precision and store it as a smaller integer?
        - negative precision where the integer value is taken to be the "step" (a.k.a. minimum precision)
    - precision of `0` is an integer.