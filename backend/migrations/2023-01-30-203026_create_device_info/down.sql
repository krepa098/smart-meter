-- This file should undo anything in `up.sql`
DROP TABLE devices;

ALTER TABLE measurements
DROP COLUMN bat_cap;

ALTER TABLE measurements
RENAME COLUMN bat_v to v_bat;

