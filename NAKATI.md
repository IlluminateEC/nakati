So, what're the design goals of Nakati?
- Secure       --- Everything should be manditorily verified before usage. (and the language should have a security scanner built in)
    - sources of insecure values must be traced through the program to make sure injections cannot occur (e.g. SQL, commands)
    - `fn sanitize(a: String): String removes UserInput`
- Strict       --- Types are strictly checked with subtyping to make it harder to make mistakes
    - no implicit conversions, but syntax should be convenient. Parenthesis shouldn't need to be used (no `(a :: B).c`, no `(a as B).c`)
    - `a.as[B].c`
- Safe         --- Achieved through garbage collection
    - Use after free, etc. should be impossible
    - as should buffer overflows. (const size buffers won't exist)
- Satisfying   --- Convenient language features to make development more fun
- Scalable     --- support horizontal scaling with decisions. abstract things sufficiently to allow for this. Provide standard interfaces to support good design practices.
    - ```
// OpenTelemetry API
import Tracing from "@nakati/telemetry";

Tracing.withSpan((span) => {
    span.addContext("idk");
    span.addAttributes({
        "name": "idk something",
        "ae": "owkgdsf",
    });
});
```

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