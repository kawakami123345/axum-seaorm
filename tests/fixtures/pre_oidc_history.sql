-- Audited schemas used free-text actor names and the original application status.
ALTER TABLE book ADD COLUMN created_at timestamptz NOT NULL, ADD COLUMN updated_at timestamptz NOT NULL,
    ADD COLUMN created_by varchar NOT NULL, ADD COLUMN updated_by varchar NOT NULL;
ALTER TABLE publisher ADD COLUMN created_at timestamptz NOT NULL, ADD COLUMN updated_at timestamptz NOT NULL,
    ADD COLUMN created_by varchar NOT NULL, ADD COLUMN updated_by varchar NOT NULL;
CREATE TABLE book_history (history_id serial PRIMARY KEY, operation_type varchar NOT NULL,
    operation_at timestamptz NOT NULL, LIKE book);
CREATE TABLE publisher_history (history_id serial PRIMARY KEY, operation_type varchar NOT NULL,
    operation_at timestamptz NOT NULL, LIKE publisher);
CREATE FUNCTION save_history_book() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO book_history SELECT nextval('book_history_history_id_seq'), TG_OP, now(), (OLD).*;
    ELSE
        INSERT INTO book_history SELECT nextval('book_history_history_id_seq'), TG_OP, now(), (NEW).*;
    END IF;
    RETURN NULL;
END $$;
CREATE FUNCTION save_history_publisher() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO publisher_history SELECT nextval('publisher_history_history_id_seq'), TG_OP, now(), (OLD).*;
    ELSE
        INSERT INTO publisher_history SELECT nextval('publisher_history_history_id_seq'), TG_OP, now(), (NEW).*;
    END IF;
    RETURN NULL;
END $$;
CREATE TRIGGER trigger_book_history AFTER INSERT OR UPDATE OR DELETE ON book FOR EACH ROW EXECUTE FUNCTION save_history_book();
CREATE TRIGGER trigger_publisher_history AFTER INSERT OR UPDATE OR DELETE ON publisher FOR EACH ROW EXECUTE FUNCTION save_history_publisher();
INSERT INTO publisher (pub_id, name, created_at, updated_at, created_by, updated_by)
VALUES (gen_random_uuid(), 'Original', now(), now(), 'old operator', 'old operator');
INSERT INTO book (pub_id, title, author, publisher_id, status, price, created_at, updated_at, created_by, updated_by)
VALUES (gen_random_uuid(), 'Original', 'A', 1, 'Applied', 1000, now(), now(),
        '00000000-0000-0000-0000-000000000001', 'old operator');
INSERT INTO seaql_migrations VALUES ('m20260101_000002_add_audit_and_history', 0);
