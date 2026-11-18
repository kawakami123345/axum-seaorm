-- Some installations applied the original book/publisher-only first migration.
-- This runs in the migration transaction before the revision guards need shop.
CREATE TABLE IF NOT EXISTS shop (
    id serial PRIMARY KEY, pub_id uuid NOT NULL UNIQUE, name varchar(32) NOT NULL UNIQUE,
    created_at timestamptz NOT NULL, updated_at timestamptz NOT NULL,
    created_by uuid NOT NULL, updated_by uuid NOT NULL
);

ALTER TABLE book DISABLE TRIGGER trigger_book_history;
ALTER TABLE publisher DISABLE TRIGGER trigger_publisher_history;
ALTER TABLE book ADD COLUMN IF NOT EXISTS shop_id integer REFERENCES shop(id);

DO $$
DECLARE
    tbl text;
    actor text;
    actors text[];
    legacy_col text;
    unknown_actor constant uuid := '00000000-0000-0000-0000-000000000000';
BEGIN
    FOREACH tbl IN ARRAY ARRAY['publisher', 'publisher_history', 'book', 'book_history', 'shop'] LOOP
        EXECUTE format('ALTER TABLE %I
            ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
            ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP', tbl);
        actors := ARRAY['created_by', 'updated_by'];
        IF tbl IN ('book', 'book_history') THEN
            actors := array_append(actors, 'user_id');
        END IF;
        IF tbl IN ('book_history', 'publisher_history') THEN
            actors := array_append(actors, 'operation_by');
        END IF;
        FOREACH actor IN ARRAY actors LOOP
            IF EXISTS (SELECT FROM pg_attribute WHERE attrelid = tbl::regclass
                       AND attname = actor AND NOT attisdropped AND atttypid <> 'uuid'::regtype) THEN
                -- Retain free-text actor names from versions predating OIDC.
                legacy_col := 'legacy_' || actor;
                EXECUTE format('ALTER TABLE %I RENAME COLUMN %I TO %I', tbl, actor, legacy_col);
                EXECUTE format('ALTER TABLE %I ALTER COLUMN %I DROP NOT NULL', tbl, legacy_col);
                EXECUTE format('ALTER TABLE %I ADD COLUMN %I uuid NOT NULL DEFAULT %L', tbl, actor, unknown_actor);
                EXECUTE format('UPDATE %I SET %I = %I::uuid WHERE pg_input_is_valid(%I::text, ''uuid'')',
                               tbl, actor, legacy_col, legacy_col);
            ELSE
                EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS %I uuid NOT NULL DEFAULT %L',
                               tbl, actor, unknown_actor);
            END IF;
        END LOOP;
    END LOOP;

    IF EXISTS (SELECT FROM pg_attribute WHERE attrelid = 'book'::regclass AND attname = 'status') THEN
        -- The earliest version accepted 128-character titles. New audit rows
        -- must also accommodate these titles after an update or deletion.
        ALTER TABLE book_history ALTER COLUMN title TYPE text;
    END IF;
    FOREACH tbl IN ARRAY ARRAY['book', 'book_history'] LOOP
        EXECUTE format('ALTER TABLE %I
            ADD COLUMN IF NOT EXISTS shop_id integer,
            ADD COLUMN IF NOT EXISTS applied_at timestamptz,
            ADD COLUMN IF NOT EXISTS format varchar(32) NOT NULL DEFAULT ''Real'',
            ADD COLUMN IF NOT EXISTS applied_without_date boolean NOT NULL DEFAULT false', tbl);
        IF EXISTS (SELECT FROM pg_attribute WHERE attrelid = tbl::regclass AND attname = 'status') THEN
            EXECUTE format('ALTER TABLE %I RENAME COLUMN status TO legacy_status', tbl);
            EXECUTE format('ALTER TABLE %I ALTER COLUMN legacy_status DROP NOT NULL', tbl);
            EXECUTE format('UPDATE %I SET applied_without_date = true
                            WHERE legacy_status = ''Applied'' AND applied_at IS NULL', tbl);
        END IF;
    END LOOP;
END $$;

-- The first publisher table allowed duplicate names. Retain every old public
-- ID and book reference, while enforcing uniqueness on new and renamed rows.
DO $$
BEGIN
    IF to_regclass('publisher_name_key') IS NOT NULL THEN
        RETURN;
    END IF;
    IF NOT EXISTS (SELECT FROM publisher GROUP BY name HAVING count(*) > 1) THEN
        ALTER TABLE publisher ADD CONSTRAINT publisher_name_key UNIQUE (name);
        RETURN;
    END IF;

    ALTER TABLE publisher ADD COLUMN legacy_duplicate_name boolean NOT NULL DEFAULT false;
    UPDATE publisher SET legacy_duplicate_name = true WHERE id IN (
        SELECT id FROM (SELECT id, row_number() OVER (PARTITION BY name ORDER BY id) AS n FROM publisher) ranked
        WHERE n > 1
    );
    CREATE UNIQUE INDEX publisher_name_key ON publisher (name) WHERE NOT legacy_duplicate_name;
    CREATE OR REPLACE FUNCTION guard_legacy_publisher_name() RETURNS trigger AS $fn$
    BEGIN
        IF TG_OP = 'UPDATE' AND NEW.name = OLD.name THEN RETURN NEW; END IF;
        IF EXISTS (SELECT FROM publisher WHERE legacy_duplicate_name AND name = NEW.name) THEN
            RAISE EXCEPTION 'Publisher name already exists' USING ERRCODE = '23505', CONSTRAINT = 'publisher_name_key';
        END IF;
        NEW.legacy_duplicate_name := false;
        RETURN NEW;
    END;
    $fn$ LANGUAGE plpgsql;
    CREATE TRIGGER check_legacy_publisher_name BEFORE INSERT OR UPDATE OF name ON publisher
    FOR EACH ROW EXECUTE FUNCTION guard_legacy_publisher_name();
END $$;
