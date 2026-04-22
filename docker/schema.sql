SELECT * 
FROM pg_create_physical_replication_slot('standby1', TRUE);

CREATE ROLE repl_user LOGIN REPLICATION ENCRYPTED PASSWORD 'password';

CREATE TABLE IF NOT EXISTS users (
	id SERIAL  PRIMARY KEY,
	name TEXT NOT NULL,
	age INT NOT NULL
);

CREATE TABLE IF NOT EXISTS posts (
	id SERIAL PRIMARY KEY,
	user_id INT REFERENCES users(id),
	content TEXT NOT NULL,
	created_at DATE NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS organizations (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL
);
