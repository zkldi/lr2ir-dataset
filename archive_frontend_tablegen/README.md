# LR2IR Archive Frontend Tablegen

Generate a `tableinfo.db` file for use alongside the LR2IR Archive Frontend.

## Usage

```
cargo run -p archive_frontend_tablegen
```

This will fetch tables defined in `src/main.rs` and spit out a `tableinfo.db` file.

You can then configure the frontend to read this file!
