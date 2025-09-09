# Nakati
Hello, Nakati will be a programming language that is meant to fulfil a few goals (and Kazani has arbitrarily restricted those goals to all start with S):
Nakati should be:
- Satisfying: it should be convenient and pleasant to use!!
    - Easy / convenient ways to implement features should be prioritized over other ways
    - Things should not be required to be overly verbose (that is Kazani's personal choice to write their code with 5 word method names)
    - Syntax should be convenient to type, and convenient to continue. (e.g. it is harder to use the return value of `await someFunction()` than `someFunction().await`, so the latter should be preferred)
- Strict: types should be strictly checked to reduce mistakes
    - Static subtyping to allow clarifying ranges of numbers, that specific kinds of numbers are different from others
    - Implicit conversions should not be used excessively (i.e. like JavaScript). Basic conversions for primitive types should be done for convenience, though, when it cannot cause errors (thus, no implicit type narrowing).
    - No gradual typing. Sorry. That complicates type systems too much.
- Safe: memory safety should be achieved
    - Garbage collection usually resolves most memory safety issues
    - Fixed size buffers aren't really necessary (and usually cause vulnerabilities if unchecked)
    - Perhaps if the compiler can determine lifetimes for specific allocations, it should put manual deallocations in instead of using garbage collection.
- Secure: everything should be manditorily verified before use
    - Vague ideas for now, but user inputs should never be blindly trusted
    - User inputs should be required to be converted into more specific types, or have "tainted attributes" removed (like "UserInput")
    - Perhaps someway to encode that something has been verified?
    - Try to avoid SQL injections and shell injections
    - Security scanning should be done as a part of this goal
- Scalable: it should make it easy to horizontally scale backend services
    - Actor model for concurrency, like Elixir
    - OpenTelemetry for logging
    - Sufficient abstraction for networking that it is easy to switch from a intra-process call to a inter-process call (perhaps even transparently)
    - Standard library should promote good design practices with open standards.

Now, do any of these features exist yet? No. Come back later. Or write it for us, /shrug. (We would really appreciate that.)
Hopefully this doesn't become abandoned programming language number 193481 on GitHub.