- figure out GC
- figure out bytecode compiler
- figure out semantics for primitives being referenced by a closure
- write the error system
- builtins should probably be handled by the import system
- variadic functions?

## GC
- Reference notation makes something garbage collected (`&Type` vs. `Type`)
- Otherwise, it's passed by move semantics
- The compiler should be able to figure out when a pointer is required for a `Type`, like when it is generic
- The compiler should be able to collapse a reference to a pointer to a type into just a reference.
    (e.g. `&(String | u128)`)
    This would require automatic type tagging...
        But the language can usually fit that into the unused space of the pointer (at least on 64-bit architectures. 32-bit and below will probably be quite inefficient.)
- Maybe references don't always have to be garbage collected? As in, if you reference an array element it'll be a raw pointer?
    Dunno. That complicates things. It should probably just be an array of references if that's needed? (well, no, not really????)
    I could have pointers... I don't want to have pointers but they would be helpful. I should figure out how to avoid pointers.

## Types
- `extend Class with ParentClass {}` (ext) also valid. Inheritance only valid for classes.
    Consider:
        It's not really extending Class with ParentClass, it's that Class is an extension of ParentClass, so it's more that Class extends ParentClass, not that ParentClass extends Class.
        Perhaps figure out different syntax ^
- `implement Interface for Type {}` (impl) also valid
- `implement Type {}` (impl also valid)
    Consider alternative keywords:
        - `behavior`
        - `methods`
        - only `impl` (short for implementation instead of implement, here)
- `class Class {}`
- `type Type;` (marker type)
- `type Type = AnotherType;` (type alias)
- `subtype Type = AnotherType;` (subtypes)
    Consider alternative syntax:
        - `type Type := AnotherType;`
    Consider:
        This should not be usable as an `AnotherType`, but the methods should be accessible.
        Thus, `subtype`s (definition) are not `subtype`s (operator).
            This is confusing. Find a different name.
- `union Type = Variant() | Variant {} | Variant;` (algebraic data types)
    Consider alternative keywords:
        - `data`
        - `enum`
        - `tagged`
    - ```
tagged Result[T, E]
    = Ok(T)
    | Err(E)
    ;
```
    - ```
tagged Optional[T] =
    | Some(T)
    | None
    ;

tagged Optional[T] =
    | Some(T)
    | None;

tagged Optional[T] 
    = Some(T)
    | None;

tagged Optional[T] 
    = Some(T)
    | None
    ;
```
    Tagged unions should be implemented such that each variant is its own type that inherits from the main type, allowing you to typehint for a specific variant rather than the entire union. e.g.
    ```
tagged Ast =
    | BinOp(String, Ast, Ast)
    | Literal(u127)
    ;
```

## Tagged functions / Function macros
- `fn#server listUsers() -> Vector[User];`
- `fn#client renderUsers() -> DOMBlock;`
- Some effects won't be propagatable across server-client boundaries
    - e.g. getDatabase
    - Would need to get effect handler from the server side call stack instead?
- TODO: figure out how this could ever possibly work
- Sounding like an annoying registry system
- qhar