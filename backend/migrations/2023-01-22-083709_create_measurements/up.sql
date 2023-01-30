-- Your SQL goes here
CREATE TABLE measurements (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    device_id INTEGER NOT NULL,
    timestamp BIGINT NOT NULL,
    temperature REAL,
    humidity REAL,
    pressure REAL,
    air_quality REAL,
    v_bat REAL
);
