# llvm-kaleidoscope-rs

The purpose of this repository is to learn about the [`llvm`][llvm] compiler
infrastructure and practice some [`rust-lang`][rust].

To reach the goals set, we follow the official llvm tutorial [`Kaleidoscope:
Implementing a Language with LLVM`][llvm-tutorial]. This tutorial is written in
`C++` and structured in multiple chapters, we will try to follow along and
implement every chapter in rust.

The topics of the chapters are as follows:

- Chapter 1: [Kaleidoscope Introduction and the Lexer][llvm-ch1]
- Chapter 2: [Implementing a Parser and AST][llvm-ch2]
- Chapter 3: [Code generation to LLVM IR][llvm-ch3]
- Chapter 4: [Adding JIT and Optimizer Support][llvm-ch4]
- Chapter 5: [Extending the Language: Control Flow][llvm-ch5]

The implementation after each chapter can be compiled and executed by checking
out the corresponding tag for the chapter.
```bash
> git tag -l
chapter1
chapter2
chapter3
chapter4
chapter5
```

Names of variables and functions as well as the structure of the functions are
mainly kept aligned with the official tutorial. This aims to make it easy to
map the `rust` implementation onto the `C++` implementation when following the
tutorial.

One further note on the llvm API, instead of using the llvm `C++` API we are
going to use the llvm `C` API and build our own safe wrapper specialized for
this tutorial. The wrapper offers a similar interface as the `C++` API and is
implemented in [`src/llvm/`](src/llvm/)

## Demo

```bash
# Run kaleidoscope program from file.
cargo run ks/<file>

# Run REPL loop, parsing from stdin.
cargo run
```

## Documentation

Rustdoc for this crate is available at
[johannst.github.io/llvm-kaleidoscope-rs][gh-pages].

## Build with provided container file

The provided [Dockerfile](docker/Dockerfile) documents the required
dependencies for an ubuntu based system and serves as a build environment with
the correct llvm version as specified in the [Cargo.toml](Cargo.toml) file.

```bash
# Build the image *ks-rs*. Depending on the downlink this may take some minutes.
make -C docker

podman run --rm -it -v $PWD:/work -w /work ks-rs
# Drops into a shell in the container, just use cargo build / run ...
```

## License

This project is licensed under the [MIT](LICENSE) license.

[llvm]: https://llvm.org
[llvm-tutorial]: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html
[llvm-ch1]: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl01.html
[llvm-ch2]: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl02.html
[llvm-ch3]: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl03.html
[llvm-ch4]: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl04.html
[llvm-ch5]: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl05.html
[rust]: https://www.rust-lang.org
[gh-pages]: https://johannst.github.io/llvm-kaleidoscope-rs/llvm_kaleidoscope_rs/index.html
