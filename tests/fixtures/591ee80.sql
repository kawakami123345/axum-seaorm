-- Original first migration, before shop, audit columns or ownership existed.
CREATE TABLE publisher (id serial PRIMARY KEY, pub_id uuid NOT NULL UNIQUE, name varchar NOT NULL);
CREATE TABLE book (
    id serial PRIMARY KEY, pub_id uuid NOT NULL UNIQUE, title varchar NOT NULL,
    author varchar NOT NULL, publisher_id integer NOT NULL REFERENCES publisher(id),
    status varchar NOT NULL, price integer NOT NULL
);
CREATE TABLE seaql_migrations (version varchar PRIMARY KEY, applied_at bigint NOT NULL);
INSERT INTO seaql_migrations VALUES ('m20220101_000001_create_table', 0);
