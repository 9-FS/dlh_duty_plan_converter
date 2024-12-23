CREATE TABLE Airport
(
    id INTEGER NOT NULL,
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
    keywords TEXT,
    PRIMARY KEY(id)
);

CREATE TABLE Country
(
    id INTEGER NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    continent TEXT NOT NULL,
    wikipedia_link TEXT,
    keywords TEXT,
    PRIMARY KEY(id)
);