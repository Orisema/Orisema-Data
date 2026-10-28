# Banco de dados

## Separação confirmada

O banco interno do Orisema Data continua sendo PostgreSQL, acessado por pool SQLx via `RUSTDB_DATABASE_URL`. Bancos de destino podem ser PostgreSQL, MySQL, MariaDB ou SQL Server e são sempre acessados por conexões dedicadas. Não usar banco valioso em testes.

## Schema interno

- `0001_connections.sql`: conexões e credencial cifrada; originalmente restringe o tipo a PostgreSQL.
- `0002_tls.sql`: `ssl_mode` em `verify_full` ou `disable`.
- `0003_security.sql`: usuários, sessões, auditoria e políticas por conexão.
- `0004_multidriver.sql`: substitui somente o CHECK de `database_type`, aceitando `postgresql`, `mysql`, `mariadb` e `sqlserver`.

Nenhuma migration altera o ciphertext. Senhas de destino continuam em AES-256-GCM, nonce aleatório, UUID como AAD e formato `v1:`. Respostas nunca retornam senha ou ciphertext.

## Destinos

Todos os drivers usam TLS verificado por padrão, parâmetros vinculados nos filtros de metadados, limite de 10.000 itens e consultas dedicadas. O contrato cobre schemas, tabelas/views e colunas/tipos; chaves e índices: A confirmar após extensão explícita do contrato.

Cancelamento em MySQL, MariaDB e SQL Server derruba a conexão dedicada e é best-effort. PostgreSQL também possui statement timeout. Privilégios mínimos e controles de recurso no servidor de destino continuam necessários.

Rotação de chave, rollback formal de migrations, backup e retenção: A confirmar.
