# LR2IR Archive Parser

Parses the data in `lr2ir-raw-unprocessed-data.zip` into the `lr2ir-archive.db` file.

Runs with high parallelism, so compile in `--release`.

## Usage

```sh
# release is critically important, it is far too slow otherwise
# configure --pages-dir and --db to taste.
cargo run --release -p archive_parser -- --pages-dir path/to/raw-data --db sqlite://output/lr2ir-archive.db
```
