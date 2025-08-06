CREATE TABLE Airport
(
    id INTEGER PRIMARY KEY,
    ident TEXT NOT NULL,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    latitude_deg REAL NOT NULL,
    longitude_deg REAL NOT NULL,
    elevation_ft INTEGER,
    continent TEXT NOT NULL,
    iso_country TEXT NOT NULL,
    iso_region TEXT NOT NULL,
    municipality TEXT,
    scheduled_service BOOLEAN NOT NULL,
    gps_code TEXT,
    iata_code TEXT,
    local_code TEXT,
    home_link TEXT,
    wikipedia_link TEXT,
    keywords TEXT
);

CREATE TABLE Country
(
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    continent TEXT NOT NULL,
    wikipedia_link TEXT,
    keywords TEXT
);

CREATE TABLE Event
(
    uid TEXT PRIMARY KEY,
    summary TEXT,
    start_dt TEXT NOT NULL,
    end_dt TEXT NOT NULL,
    location TEXT,
    description TEXT
);