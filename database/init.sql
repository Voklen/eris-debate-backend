CREATE TABLE IF NOT EXISTS users(
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	email TEXT NOT NULL UNIQUE,
	password_hash TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS session_tokens(
	id BIGINT NOT NULL REFERENCES users(id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	token BYTEA NOT NULL
);
CREATE TABLE IF NOT EXISTS arguments(
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	created_by BIGINT NOT NULL REFERENCES users(id),
	parent BIGINT REFERENCES arguments(id),
	body TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS topics(
	name TEXT NOT NULL UNIQUE,
	id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	for_argument BIGINT NOT NULL REFERENCES arguments(id),
	against_argument BIGINT NOT NULL REFERENCES arguments(id)
);
