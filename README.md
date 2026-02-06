# Typst PlantUML

A simple tool that detects changes to Typst code and uses inline PlantUML to generate diagrams.

## Warnings

This is incredibly unstable and rush software.
It only exists to aid some very specific use cases which has resulted in a severe lack of flexibility.
In other words, the interface and code has been thrown together with a lot of undocumented assumptions about the systems it will run on and patterns of usage.

## Usage

With all the warnings out of the way, usage is pretty simple.

On most systems, you can simply install via cargo:

```bash
cargo install --path . --locked
```

Then just run with `typst-plantuml <some directory>`.

Nix support will come very soon.
