CREATE TABLE Event
(
    uid TEXT NOT NULL,
    summary TEXT,
    start_dt TEXT NOT NULL,
    end_dt TEXT NOT NULL,
    location TEXT,
    description TEXT,
    PRIMARY KEY(uid)
);