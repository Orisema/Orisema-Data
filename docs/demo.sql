-- Run only in a disposable demo database, never in the internal configuration database.
CREATE TABLE IF NOT EXISTS produtos (
    codigo INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    descricao TEXT NOT NULL,
    preco NUMERIC(10,2),
    ativo BOOLEAN NOT NULL DEFAULT true
);
INSERT INTO produtos (descricao,preco,ativo) VALUES
    ('Teclado mecânico',249.90,true),
    ('Monitor 27 polegadas',1299.00,true),
    ('Mouse sem fio',89.50,true),
    ('Adaptador USB-C',NULL,false);
CREATE OR REPLACE VIEW produtos_ativos AS SELECT * FROM produtos WHERE ativo;
