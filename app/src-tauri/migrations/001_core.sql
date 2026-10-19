PRAGMA foreign_keys=ON;
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
INSERT OR IGNORE INTO meta VALUES ('schema_version','1');
CREATE TABLE IF NOT EXISTS exam_runs (id TEXT PRIMARY KEY, paper_id TEXT NOT NULL, paper_version INTEGER NOT NULL, status TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS attempts (id TEXT PRIMARY KEY, paper_id TEXT NOT NULL, paper_version INTEGER NOT NULL, section TEXT NOT NULL, mode TEXT NOT NULL, status TEXT NOT NULL, revision INTEGER NOT NULL, snapshot TEXT NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS responses (attempt_id TEXT NOT NULL REFERENCES attempts(id), question_id TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY(attempt_id,question_id));
CREATE TABLE IF NOT EXISTS annotations (attempt_id TEXT NOT NULL REFERENCES attempts(id), ordinal INTEGER NOT NULL, value TEXT NOT NULL, PRIMARY KEY(attempt_id,ordinal));
CREATE TABLE IF NOT EXISTS interruption_events (attempt_id TEXT NOT NULL REFERENCES attempts(id), ordinal INTEGER NOT NULL, value TEXT NOT NULL, PRIMARY KEY(attempt_id,ordinal));
