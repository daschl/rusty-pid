# How To Flash

## During Development

If using the development kit for testing use:

```
cargo run --features board-dk
```

If using the j-link directly with the bluefruit board, use:

```
cargo run --features board-bluefruit
```

## For Production

Since `cargo run` always starts the interactive terminal with the `defmt` output, I'm using `cargo flash` directly to flash the firwmare onto the board.

```
cargo flash --release --features board-bluefruit --chip nRF52840_xxAA
```

Version used: `cargo-flash 0.9.0`