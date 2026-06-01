# LR2IR Archive Frontend

A simple HTML frontend for navigating and browsing the LR2IR dataset.

This is a completely read-only archive, and just lets people browse the data we've backed up.

## Usage

### For local development

Install [rustup](https://rustup.sh)

Then install [cargo-watch](https://github.com/watchexec/cargo-watch) and [just](https://just.systems):

Then run `just frontend`.

### For real hosting

Firstly, copy the `lr2ir-archive.db` and `tableinfo.db` files to your server.

If you want to host this yourself you have two options.

#### Docker

We provide a docker image on ghcr.io:

```sh
docker run --rm -p 3000:3000 \
	-v "path/to/your/lr2ir-archive.db:/data/lr2ir-ar chive.db:ro" \
	-v "path/to/your/tableinfo.db:/data/tableinfo.db:ro" \
	ghcr.io/zkldi/lr2ir-dataset/archive-frontend:latest
```

#### Compile it yourself

Alternatively, self-compile it like this.

```
cargo build --release -p archive_frontend
```

Then copy the file from `target/release/lr2ir_archive_frontend` to a server of your choice.

You can run the binary like this:

```sh
./lr2ir_archive_frontend serve --bind 0.0.0.0:3000 --tableinfo path/to/tableinfo.db --database path/to/lr2ir-archive.db
```

or you can use the `TABLEINFO_PATH` and `DATABASE_PATH` env vars.

### Proxying

With it running on your server on a port (3000, or whatever) you will need a reverse
proxy to expose it to the internet correctly.

I recommend installing [caddy](https://caddyserver.com/) and using that. Your caddy config will look like this:

```caddy
# Replace example.com with your domain
example.com {
    reverse_proxy localhost:3000
    encode zstd gzip
    # Optional: enable automatic HTTPS
    tls your@email.com
}
```
