-- Commit device acknowledgements with the archived message so cleanup can resume
-- after a restart or a partially successful multipart deletion.
CREATE TABLE modem_sms_receipts (
    sim_id TEXT NOT NULL,
    storage TEXT NOT NULL,
    modem_index INTEGER NOT NULL,
    pdu TEXT NOT NULL,
    PRIMARY KEY (sim_id, storage, modem_index)
);
