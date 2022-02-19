# brainfuck

An implementation of [Brainfuck](https://esolangs.org/wiki/Brainfuck) written in Rust.

In this implementation, the array of memory cells is 30,000 long, like the original. Attempting to
move the data pointer outside of the bounds of the array will result in a panic.

## Usage

Build using Cargo.

Can be run as a REPL or with a `.bf` file.

```bash
$ brainfuck [file [-v] [-c]]
```

* `-c` enables displaying output as characters rather than byte values.
* `-v` enables verbose mode for the compiler (will output compilation time).

While in the REPL, entering "exit" will quit the REPL, and any characters that are not `<`, `>`, `+`, `-`, `[`, or `]` are ignored.
