# Vector From Scratch

A dynamic vector implemented from scratch in Rust.

This project was made to learn how `unsafe` Rust works and how dynamic data structures manage memory under the hood.

The vector uses raw pointers and manual heap allocation to implement things like:

* `push` and `pop`
* `insert` and `remove`
* automatic capacity growth
* indexing
* iteration
* manual memory cleanup

The goal is **not** to build a better `Vec<T>` than Rust's standard library, but to understand what happens behind the scenes when building a dynamic array.

## Structure

```text
src/
├── main.rs
└── vector.rs
```

* `vector.rs` — implementation of the vector
* `main.rs` — small example using it

## Run

```bash
cargo run
```

Run the tests with:

```bash
cargo test
```

> This is an educational project and is not intended for production use.
