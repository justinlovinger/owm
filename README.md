[![Workflow Status](https://github.com/justinlovinger/owm/workflows/build/badge.svg)](https://github.com/justinlovinger/owm/actions?query=workflow%3A%22build%22)

# owm

An experimental [River](https://github.com/riverwm/river) layout generator
using mathematical optimization
to invent layouts
on-the-fly.

## Building

Run `cargo build --release` or `nix build`.

## Usage

Add

```
riverctl default-layout owm
owm &
```

to your River `init`.

See `owm --help` for configuration options.
