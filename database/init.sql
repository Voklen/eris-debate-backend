CREATE TABLE IF NOT EXISTS users(
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	email TEXT NOT NULL UNIQUE,
	email_verified BOOLEAN NOT NULL DEFAULT false,
	username TEXT NOT NULL UNIQUE,
	password_hash TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS session_tokens(
	id BIGINT NOT NULL REFERENCES users(id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	token_hash TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS revisions(
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	revision_by BIGINT NOT NULL REFERENCES users(id),
	body TEXT NOT NULL,
	prev_revision BIGINT REFERENCES revisions(id)
);
CREATE TABLE IF NOT EXISTS arguments(
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	revision_latest BIGINT NOT NULL REFERENCES revisions(id),
	parent BIGINT REFERENCES arguments(id)
		ON DELETE CASCADE
		ON UPDATE CASCADE
);
CREATE TABLE IF NOT EXISTS topics(
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name TEXT NOT NULL UNIQUE,
	for_argument BIGINT NOT NULL REFERENCES arguments(id),
	against_argument BIGINT NOT NULL REFERENCES arguments(id)
);
CREATE TABLE IF NOT EXISTS topic_proposals(
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name TEXT NOT NULL,
	created_by BIGINT NOT NULL REFERENCES users(id),
	for_argument TEXT,
	against_argument TEXT,
	reason TEXT
);
CREATE TABLE IF NOT EXISTS roles(
	id BIGINT NOT NULL REFERENCES users(id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	role TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS email_verification_tokens(
	id BIGINT NOT NULL REFERENCES users(id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	token TEXT NOT NULL
);
