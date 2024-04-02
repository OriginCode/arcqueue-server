CREATE EXTENSION pg_trgm;
DROP SCHEMA IF EXISTS arcqueue CASCADE;
CREATE SCHEMA arcqueue;

-- Arcades Table
CREATE TABLE arcqueue.arcades (
	id		int PRIMARY KEY,
	name		text NOT NULL,
	description	text,
	create_date	date NOT NULL
);

-- Cabinets Table
CREATE TABLE arcqueue.cabinets (
	id		int PRIMARY KEY,
	game_name	text NOT NULL,
	name		text NOT NULL,
	description	text,
	assoc_arcade	int REFERENCES arcqueue.arcades(id)
);

-- Players Table
CREATE TABLE arcqueue.players (
	position	int,
	name		text NOT NULL,
	assoc_cabinet	int REFERENCES arcqueue.cabinets(id)
);
