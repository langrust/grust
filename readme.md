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
Need nightly or the Toolchain
> cargo build 
[...]

To run without macro diagnostics
Doesn't need nightly or the Toolchain
> cargo build --features "compiler_top/no_diagnostics"
[...]

> cargo test
[...]
```

