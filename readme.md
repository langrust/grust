# grust

A procedural macro for the grust language.


## Build

Retrieve the creusot dependency:

```text
> git submodule update --init
```

Use cargo to build everything:

```text
To run with macro diagnostics
> cargo build
[...]

To run without macro diagnostics
> cargo build --features "compiler_top/no_diagnostics compiler_common/no_diagnostics"
[...]

> cargo test
[...]
```

