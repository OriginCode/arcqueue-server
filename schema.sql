CREATE EXTENSION pg_trgm;
DROP SCHEMA IF EXISTS arcqueue CASCADE;
CREATE SCHEMA arcqueue;

-- Arcades Table
CREATE TABLE arcqueue.arcades (
	id			uuid PRIMARY KEY,
	name		text NOT NULL,
	description	text,
	create_date	date NOT NULL
);

-- Cabinets Table
CREATE TABLE arcqueue.cabinets (
	id				uuid PRIMARY KEY,
	game_name		text NOT NULL,
	name			text NOT NULL,
	assoc_arcade	uuid REFERENCES arcqueue.arcades (id)
);

-- Players Table
CREATE TABLE arcqueue.players (
	position		int NOT NULL,
	name			text NOT NULL,
	assoc_cabinet	uuid REFERENCES arcqueue.cabinets (id),
	UNIQUE (position, assoc_cabinet),
	UNIQUE (name, assoc_cabinet)
);
