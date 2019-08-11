# ⚛️ gargantua

A blackhole (Schwardzchild spacetime) raytracer, written in Rust.

This project is composed of several crates, separating concerns of the different parts of the project.

# Install & Run

To install, you must have the latest stable version of Rust on your computer, as well as Cargo (both included as part of [Rustup](https://rustup.rs/)). Then, simply run `cargo run` to run the program. 

## Command line arguments

```
gargantua (now Rusty!) 0.1
Nathan Graule <solarliner@gmail.com>
Render black hole in Flat (boring) or Schwardzchild (awesome) spacetime.

USAGE:
    gargantua-bin [FLAGS] [OPTIONS] [OUT] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -q               Quiet output (no progress readout)
    -V, --version    Prints version information

OPTIONS:
    -s <WIDTHxHEIGHT>        Sets the output image size

ARGS:
    <OUT>    Output filename

SUBCOMMANDS:
    flat      Renders a black hole in flat spacetime
    help      Prints this message or the help of the given subcommand(s)
    warped    Renders scene in Schwardzchild spacetime
```
