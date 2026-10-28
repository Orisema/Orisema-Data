ALTER TABLE connections DROP CONSTRAINT connections_database_type_check;
ALTER TABLE connections ADD CONSTRAINT connections_database_type_check
    CHECK (database_type IN ('postgresql', 'mysql', 'mariadb', 'sqlserver'));
