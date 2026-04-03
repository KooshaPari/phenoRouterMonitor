-- UP
CREATE TABLE IF NOT EXISTS wp_dependencies (
    wp_id      INTEGER NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    depends_on INTEGER NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    dep_type   TEXT    NOT NULL CHECK(dep_type IN ('explicit','file_overlap','data')),
    PRIMARY KEY (wp_id, depends_on)
);

-- DOWN
DROP TABLE IF EXISTS wp_dependencies;
