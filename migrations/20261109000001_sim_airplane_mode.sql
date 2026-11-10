-- Persist the desired radio state per SIM so a line held in airplane mode does
-- not register with the network again after a service restart.
ALTER TABLE sim_cards ADD COLUMN airplane_mode INTEGER NOT NULL DEFAULT 0;
