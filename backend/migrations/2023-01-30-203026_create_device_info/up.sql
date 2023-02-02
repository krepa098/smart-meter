-- Your SQL goes here
CREATE TABLE devices (
    device_id INTEGER PRIMARY KEY NOT NULL,
    fw_version TEXT NOT NULL,
    bsec_version TEXT NOT NULL,
    wifi_ssid TEXT,
    uptime INTEGER NOT NULL,
    report_interval INTEGER NOT NULL,
    sample_interval INTEGER NOT NULL,
    last_seen BIGINT NOT NULL
);

ALTER TABLE measurements
ADD COLUMN bat_cap REAL;

ALTER TABLE measurements
RENAME COLUMN v_bat to bat_v;