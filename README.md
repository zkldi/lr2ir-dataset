# LR2IR Datasets

LR2IR is going to be shut down on May 31st 2026. This is an archive from the website as of May 31st 2026.

# Viewer/Frontend

We have also provided a viewer for the site, which we host on [https://lr2ir.com](https://lr2ir.com). The source code for the viewer is here, and you can re-host it yourself.

Take a look in the `archive_frontend` folder for more information on that.

## Full Database

**Link:** [lr2ir-archive-v3.db.gz](https://cdn.lr2ir.com/lr2ir-archive-v3.db.gz)

**Shasum:** `b8b94d9c67cf94411a71501c3533bd15877bd727fd687c060749fae29267b5cf`

This is an 8GiB sqlite database containing every score, chart, course and player on LR2IR. This is likely the data you want.

This dataset contains `25,336,942` score entries, `97,895` players, `333,362` charts, `353,699` course plays and `3,496,970` ghost replays.

<img width="1219" height="1043" alt="image" src="https://github.com/user-attachments/assets/e24f0a6d-a249-4052-beee-ba659cc6e14d" />
<img width="1226" height="1053" alt="image" src="https://github.com/user-attachments/assets/725a2d9f-0eb3-4281-9a4c-605c01ce747d" />

### Usage

- Un `gzip` the file. You can do this with `gunzip -k path/to/file`.
- Install [sqlitebrowser](https://sqlitebrowser.org/)
- You can now open the `.db` file and view all of the information.

Alternatively, use the instructions in `archive_frontend/README.md` to set up the web browser for this data.

## Raw, unprocessed data

**Link:** [lr2ir-raw-unprocessed-data-v3.zip](https://cdn.lr2ir.com/lr2ir-raw-unprocessed-data-v3.zip)

**Checksum:** `5d688beaf6d82ec0831019823432a53037a7136e16f684828f3f7ced88ef5e1f`

This is the `html` output for all of the pages we scraped on LR2IR.

Please note, the database above contains _all_ the information in these pages. This is simply the raw output if you really specifically want it.

You will need to provide the [styles.css](https://cdn.lr2ir.com/styles.css) yourself.
The shasum for `styles.css` is `4d0eb5b48f252277e8ee4d7709926939f252fc417f6a850ae812afd8f047a213`

### Usage

- Download this .zip
- Extract it
- \*\*Be aware that some of the `.html` files are gzip encoded. You will need to `gunzip` them to open them.
