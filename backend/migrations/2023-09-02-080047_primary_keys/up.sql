-- Your SQL goes here
ALTER TABLE measurements RENAME TO measurements_old;

-- change primary key to device_id and timestamp
CREATE TABLE measurements (
    device_id INTEGER NOT NULL,
    timestamp BIGINT NOT NULL,
    temperature REAL,
    humidity REAL,
    pressure REAL,
    air_quality REAL,
    bat_v REAL,
    bat_cap REAL,
    PRIMARY KEY (device_id, timestamp)
);

-- fill the new table
INSERT INTO measurements (device_id, timestamp, temperature, humidity, pressure, air_quality, bat_v, bat_cap)
    SELECT device_id, timestamp, temperature, humidity, pressure, air_quality, bat_v, bat_cap FROM measurements_old
