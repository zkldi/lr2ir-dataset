gen:
	just gen-table
	just gen-db

gen-table:
	cargo run -p lr2ir_archive_frontend_tablegen -- --output data/tableinfo.db

gen-db PATH_TO_RAW_DATA:
	cargo run -p lr2ir_archive_parser --release -- --pages-dir {{ PATH_TO_RAW_DATA }} --db sqlite://data/lr2ir-archive.db