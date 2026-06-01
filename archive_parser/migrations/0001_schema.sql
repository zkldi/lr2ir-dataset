PRAGMA application_id = 1514880051;
PRAGMA user_version = 3;

CREATE TABLE "meta" (
    version INTEGER NOT NULL,
    rendered_at TEXT NOT NULL
) STRICT;

INSERT INTO meta (version, rendered_at) VALUES (3, CURRENT_TIMESTAMP);

CREATE TABLE "chart" (
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
) STRICT;

CREATE TABLE "pb" (
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
    is_cheated  INTEGER NOT NULL CHECK (is_cheated IN (0, 1)),
    PRIMARY KEY (md5, player_id)
) STRICT;

CREATE TABLE "ghost" (
    md5 TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    ghost BLOB NOT NULL,
    PRIMARY KEY (md5, player_id)
) STRICT;

CREATE TABLE "user" (
    player_id          INTEGER PRIMARY KEY,
    name               TEXT    NOT NULL,
    dan                TEXT    NOT NULL,
    bio                TEXT    NOT NULL,
    -- NULL = fully public;
    -- 'playcount' = play counts hidden;
    -- 'full' = entire stats hidden
    privacy_level      TEXT CHECK (privacy_level IS NULL OR privacy_level IN ('playcount', 'full')),
    songs_played       INTEGER NOT NULL,
    play_count         INTEGER NOT NULL,
    fc_count           INTEGER NOT NULL,
    perfect_fc_count   INTEGER,
    hard_count         INTEGER NOT NULL,
    normal_count       INTEGER NOT NULL,
    easy_count         INTEGER NOT NULL,
    failed_count       INTEGER NOT NULL,
    is_cheater         INTEGER NOT NULL CHECK (is_cheater IN (0, 1))
);

CREATE TABLE "user_rival" (
    player_id  INTEGER NOT NULL,
    rival_id   INTEGER NOT NULL,
    rival_name TEXT    NOT NULL,
    PRIMARY KEY (player_id, rival_id)
);

-- The top 10 "most played" charts this user had.
CREATE TABLE "user_most_plays" (
    player_id    INTEGER NOT NULL,
    pos          INTEGER NOT NULL,
    bmsid        INTEGER NOT NULL,
    title        TEXT,
    clear_type   TEXT    NOT NULL,
    play_count   INTEGER,
    rank_pos     INTEGER NOT NULL,
    rank_total   INTEGER NOT NULL,
    PRIMARY KEY (player_id, pos)
);

-- The top 10 most recent plays this user made.
CREATE TABLE "user_recent_plays" (
    player_id    INTEGER NOT NULL,
    pos          INTEGER NOT NULL,
    bmsid        INTEGER NOT NULL,
    title        TEXT,
    clear_type   TEXT    NOT NULL,
    play_count   INTEGER,
    rank_pos     INTEGER NOT NULL,
    rank_total   INTEGER NOT NULL,
    PRIMARY KEY (player_id, pos)
);

-- The top 10 most recent courses this user played.
CREATE TABLE "user_recent_courses" (
    player_id    INTEGER NOT NULL,
    pos          INTEGER NOT NULL,
    course_id    INTEGER NOT NULL,
    title        TEXT,
    clear_type   TEXT    NOT NULL,
    play_count   INTEGER,
    rank_pos     INTEGER NOT NULL,
    rank_total   INTEGER NOT NULL,
    PRIMARY KEY (player_id, pos)
);

CREATE TABLE "user_bbs" (
    player_id      INTEGER NOT NULL,
    pos            INTEGER NOT NULL,
    commenter_id   INTEGER NOT NULL,
    commenter_name TEXT,
    message        TEXT,
    posted_at      TEXT    NOT NULL,
    PRIMARY KEY (player_id, pos)
);

CREATE TABLE "course" (
    course_id    INTEGER PRIMARY KEY,
    title        TEXT    NOT NULL,
    category     TEXT    NOT NULL,
    creator_id   INTEGER NOT NULL,
    creator_name TEXT    NOT NULL,
    keys         TEXT    NOT NULL,
    play_count   INTEGER NOT NULL,
    play_people  INTEGER NOT NULL,
    clear_count  INTEGER NOT NULL,
    clear_people INTEGER NOT NULL,
    fc_count     INTEGER NOT NULL,
    hard_count   INTEGER NOT NULL,
    normal_count INTEGER NOT NULL,
    easy_count   INTEGER NOT NULL,
    failed_count INTEGER NOT NULL,
    hash         TEXT,
    course_type  INTEGER
);

-- Ordered list of charts that make up a course.
CREATE TABLE "course_stage" (
    course_id INTEGER NOT NULL,
    stage     INTEGER NOT NULL,  -- 1-based
    bmsid     INTEGER,           -- sometimes null (lol?)
    label     TEXT    NOT NULL,  -- full label text from the page
    PRIMARY KEY (course_id, stage)
);

CREATE TABLE "course_ranking" (
    course_id   INTEGER NOT NULL,
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
    -- dp only; these are right handed options
    option_3    TEXT,
    option_4    TEXT,
    input       TEXT    NOT NULL,
    client      TEXT    NOT NULL,
    -- UTF8
    note        TEXT    NOT NULL,
    is_cheated  INTEGER NOT NULL CHECK (is_cheated IN (0, 1)),
    PRIMARY KEY (course_id, player_id)
);

CREATE TABLE "bbs" (
    msgid     INTEGER NOT NULL PRIMARY KEY,
    playerid  INTEGER NOT NULL,
    -- UTF8
    message   TEXT    NOT NULL,
    -- ISO8601 with timezone
    time      TEXT    NOT NULL
);
