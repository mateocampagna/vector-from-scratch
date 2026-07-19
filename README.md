
Vector From Scratch — Dynamic vector implemented in Rust
=======================================================

Project overview
----------------

This repository contains a small Rust project that implements a dynamic array (vector) from scratch. It is an educational implementation demonstrating core concepts such as:

- Manual heap allocation and deallocation
- Capacity growth strategy (reallocation)
- Element push/pop operations
- Indexing and iteration

The implementation is minimalist and intended for learning and experimentation rather than production use.

Repository layout
-----------------

- `Cargo.toml` — project manifest
- `src/main.rs` — example runner that uses the custom vector
- `src/vector.rs` — the dynamic vector implementation and its API

Build and run
-------------

Build the project with Cargo:

```bash
cargo build --release
```

Run the example (uses the implementation from `src/vector.rs`):

```bash
cargo run --release
```

Tests
-----

If unit tests are present, run them with:

```bash
cargo test
```

Usage & API
-----------

Open [src/vector.rs](src/vector.rs) to see the public API and implementation. The example in [src/main.rs](src/main.rs) shows basic usage (creating a vector, pushing elements, indexing, and popping).

Goals and learning outcomes
--------------------------

- Understand how a dynamic array manages capacity and size
- See how Rust interacts with raw pointers and manual memory management
- Learn reallocation strategies and trade-offs

Contributing
------------

This is a small educational project. Contributions should prioritize clarity and tests that demonstrate behaviors. Open an issue or submit a pull request with a focused change.

License
-------

No license file is included by default. Add a `LICENSE` file if you intend to apply an open-source license.
