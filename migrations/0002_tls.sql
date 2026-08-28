ALTER TABLE connections ADD COLUMN ssl_mode TEXT NOT NULL DEFAULT 'verify_full'
    CHECK (ssl_mode IN ('disable', 'verify_full'));
