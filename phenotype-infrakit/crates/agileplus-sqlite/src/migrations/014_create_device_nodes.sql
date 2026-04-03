-- UP
CREATE TABLE IF NOT EXISTS device_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL UNIQUE,
    tailscale_ip TEXT,
    hostname TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    sync_vector TEXT NOT NULL,
    platform_version TEXT NOT NULL,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_devices_id ON device_nodes(device_id);
CREATE INDEX IF NOT EXISTS idx_devices_hostname ON device_nodes(hostname);
CREATE INDEX IF NOT EXISTS idx_devices_lastseen ON device_nodes(last_seen);

-- DOWN
DROP TABLE IF EXISTS device_nodes;
