# llvm-kaleidoscope-rs

The purpose of this repository is to learn about the [`llvm`][llvm] compiler
infrastructure and practice some [`rust-lang`][rust].

To reach the goals set, we follow the official llvm tutorial [`Kaleidoscope:
Implementing a Language with LLVM`][llvm-tutorial]. This tutorial is written in
`C++` and structured in multiple chapters, we will try to follow along and
implement every chapter in rust.

The implementation after each chapter can be compiled and executed by checking
out the corresponding tag for the chapter.
```bash
> git tag -l
chapter1
chapter2
chapter3
```

Names of variables and functions as well as the structure of the functions are
mainly kept aligned with the official tutorial. This aims to make it easy to
map the `rust` implementation onto the `C++` implementation when following the
tutorial.

One further note on the llvm API, instead of using the llvm `C++` API we are
going to use the llvm `C` API and build our own safe wrapper specialized for
this tutorial. The wrapper offers a similar interface as the `C++` API and is
implemented in [`src/llvm.rs`](src/llvm.rs)

## Documentation

Rustdoc for this crate is available at
[johannst.github.io/llvm-kaleidoscope-rs](gh-pages).

## License

This project is licensed under the [MIT](LICENSE) license.

[llvm]: https://llvm.org
[llvm-tutorial]: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html
[rust]: https://www.rust-lang.org
[gh-pages]: https://johannst.github.io/llvm-kaleidoscope-rs/llvm_kaleidoscope_rs/index.html
