import ".just/gen.just"

[private]
default:
	@just --list

fmt:
	cargo fmt --all

check:
	cargo clippy --all-targets -- -D warnings

# Internal that data is installed
[private]
has-data-installed:
	#!/bin/bash

	if [ ! -f data/lr2ir-archive.db ]; then
		echo "You do not have the data/lr2ir-archive.db file installed!"
		exit 1
	fi

	if [ ! -f data/tableinfo.db ]; then
		echo "first time run: generating data/tableinfo.db"
		just gen-table
	fi

# Run the frontend on 0.0.0.0:3000 with hot reload (requires cargo-watch)
frontend: has-data-installed
	#!/bin/bash

	if ! command -v cargo-watch >/dev/null 2>&1; then
		echo "cargo-watch is required. Install with: cargo install cargo-watch"
		exit 1
	fi

	cargo watch -c \
		-w archive_frontend/src \
		-w archive_frontend/templates \
		-w archive_frontend/Cargo.toml \
		-x 'run -p lr2ir_archive_frontend -- serve --bind 0.0.0.0:3000 --database data/lr2ir-archive.db --tableinfo data/tableinfo.db'
