<div align="center">

# ASCII Create

A cross-platform terminal-based image-to-ASCII converter written in Rust.

<img width="1024" height="768" alt="cover-photo" src="https://github.com/user-attachments/assets/c62a480c-8749-44a6-a939-d837f0aebdc1" />

</div>

## Features

- Supports multiple image formats through the **image** crate
- Optional edge detection using the Sobel operator
- Cross-platform terminal output via the **crossterm** crate

## How to use

Compiling the project in release mode is highly encouraged, as the **image** crate is significantly slower in debug builds. As such, you should do the following:

```
cargo build --release
./target/release/ascii-create <file_path> [options]
```

To compile and run the project:

```
cargo run --release -- <file_path> [options]
```

## Options

The tool exposes several command-line options that let you control the output size, appearance, and processing behaviour. You can:

- Limit the ASCII output dimensions (width and height)
- Adjust the character aspect ratio
- Enable and tune edge detection
- Choose the image resizing filter
- Automatically scale the image to fit your terminal
- Display processing statistics for debugging or benchmarking


Make sure to use the **-h** or **--help** flags for more information on how to use the tool.
