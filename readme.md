# grust

A procedural macro for the grust language.

## Install

1. Install `rustc` and `cargo` (instructions: https://www.rust-lang.org/tools/install)

    ```text
    > curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2. Restart terminal
3. Install `protoc` (instructions: https://protobuf.dev/installation/)
    
    ```text
    > sudo apt install protobuf-compiler

4. Retrieve `grustine` repository

    ```text
    > git clone https://gitlab.isae-supaero.fr/langrust/grustine.git
    ```

## Build

Retrieve the creusot dependency:

```text
> git submodule update --init
```

Use cargo to build everything:

```text
# To run with macro diagnostics:
> cargo build
[...]

# To run without macro diagnostics nor Creusot:
# Doesn't need nightly nor provided rust-toolchain
> cargo build --no-default-features
[...]

# To run tests:
> cargo test --all
[...]
```

