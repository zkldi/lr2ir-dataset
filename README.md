# LR2IR Datasets

LR2IR is going to be shut down on May 31st 2026. This is a backup and datasets from the website as of May 29th 2026.

## Note

At the moment, this data is not accessible in a user friendly manner, especially if you are not a programmer. In the near future, we will write a read-only frontend for all LR2IR scores + hopefully users, so that the information will not be lost. 

For now, this is just the raw data dumps.

## Score+Chart Database

**Link:** [lr2ir-all-scores.db.gz](https://cdn.lr2ir.com/lr2ir-all-scores.db.gz)

**Shasum:** `81126b2db794d01eeb7db40b196133d38a70fcf28aa834baaf5911d3ca8d50ec`

This is a 4GiB sqlite database containing every score on LR2IR. This is likely the data you want.

This dataset contains `25,271,654` score entries and all of the information you would find on the LR2IR leaderboards.
It also contains `326,434` charts and all of their information that you would find on those leaderboards, too.

<img width="1219" height="1043" alt="image" src="https://github.com/user-attachments/assets/e24f0a6d-a249-4052-beee-ba659cc6e14d" />
<img width="1226" height="1053" alt="image" src="https://github.com/user-attachments/assets/725a2d9f-0eb3-4281-9a4c-605c01ce747d" />

### Usage

- Un `gzip` the file. You can do this with `gunzip -k path/to/file`.
- Install [sqlitebrowser](https://sqlitebrowser.org/)
- You can now open the `.db` file and view all of the information.

The schema is as follows:

```sql
CREATE TABLE IF NOT EXISTS chart (
    md5              TEXT    PRIMARY KEY,
    bmsid            INTEGER,
    title            TEXT     NOT NULL,
    genre            TEXT     NOT NULL,
    artist           TEXT     NOT NULL,
    bpm_min          TEXT     NOT NULL,
    bpm_max          TEXT     NOT NULL,
    level            TEXT     NOT NULL,
    keys             TEXT     NOT NULL,
    judge_rank       TEXT     NOT NULL,
    play_count       INTEGER  NOT NULL,
    play_people      INTEGER  NOT NULL,
    clear_count      INTEGER  NOT NULL,
    clear_people     INTEGER  NOT NULL,
    fc_count         INTEGER  NOT NULL,
    hard_count       INTEGER  NOT NULL,
    normal_count     INTEGER  NOT NULL,
    easy_count       INTEGER  NOT NULL,
    failed_count     INTEGER  NOT NULL,
    last_updated_by  TEXT,
    last_updated_at  TEXT,
    body_url         TEXT,
    diff_url         TEXT,
    comment          TEXT,
    tag_1            TEXT,
    tag_2            TEXT,
    tag_3            TEXT,
    tag_4            TEXT,
    tag_5            TEXT,
    tag_6            TEXT,
    tag_7            TEXT,
    tag_8            TEXT,
    tag_9            TEXT,
    tag_10           TEXT,
    suspended        INTEGER  NOT NULL
);

CREATE TABLE IF NOT EXISTS pb (
    md5         TEXT    NOT NULL,
    rank        INTEGER NOT NULL,
    player_id   INTEGER NOT NULL,
    player_name TEXT    NOT NULL,
    dan         TEXT    NOT NULL,
    clear_type  TEXT    NOT NULL,
    letter_rank TEXT    NOT NULL,
    score       INTEGER NOT NULL,
    score_max   INTEGER NOT NULL,
    combo       INTEGER NOT NULL,
    combo_max   INTEGER NOT NULL,
    bad_poor    INTEGER NOT NULL,
    pgreat      INTEGER NOT NULL,
    great       INTEGER NOT NULL,
    good        INTEGER NOT NULL,
    bad         INTEGER NOT NULL,
    poor        INTEGER NOT NULL,
    option_1    TEXT    NOT NULL,
    option_2    TEXT    NOT NULL,
    option_3    TEXT,
    option_4    TEXT,
    input       TEXT    NOT NULL,
    client      TEXT    NOT NULL,
    note        TEXT    NOT NULL,
    PRIMARY KEY (md5, player_id)
);
```



## Raw sabun htmls

**Link:** [lr2ir-all-sabun-html.zip](https://cdn.lr2ir.com/lr2ir-all-sabun-html.zip)

**Checksum:** `e7f0943c27c533468ced5f17e2d5311a7e3b9165e8f3fdfb814e6565838e864b`

This is the `html` output for every sabun on LR2IR. Please note, the database above contains _all_ the information in these pages.
This is simply the raw output if you really specifically want it.

You will need to provide the [styles.css](https://cdn.lr2ir.com/styles.css) yourself.
The shasum for `styles.css` is `4d0eb5b48f252277e8ee4d7709926939f252fc417f6a850ae812afd8f047a213`

### Usage

- Download this .zip
- Extract it
- **Be aware that the `.html` files are gzip encoded. You will need to `gunzip` them to open them
- I have tested that firefox will natively handle and understand `gzip` encoded html files. I don't know about Chrome.

# Notes

There are no user pages yet. We were scraping them, but some third party has been effectively DDOSing the site since Saturday Morning in Japan. I want to get this data, and we did manage to get some of it, but not all.

Replay data is not here yet, but we have got top 30 replays for every chart in Insane1 and Overjoy. Again, we would like to get more, but that's just not happening.

Course data was planned too, but the site is still currently DDOSed and unusable.

