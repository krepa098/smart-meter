-- This file should undo anything in `up.sql`
DROP TABLE measurements;
ALTER TABLE measurements_old RENAME TO measurements;