# Ambiente

Rust stable (edição 2024), Node.js 24/npm, PostgreSQL 17 interno e Docker ou Podman continuam sendo os requisitos. MySQL 8.4, MariaDB 11.8 e SQL Server 2022 são usados somente em integrações descartáveis dos respectivos drivers.

Variáveis públicas do serviço: `RUSTDB_DATABASE_URL`, `RUSTDB_BOOTSTRAP_TOKEN`, `RUSTDB_ENCRYPTION_KEY`, `RUSTDB_BIND_ADDRESS`, `RUSTDB_PORT`, `RUSTDB_CORS_ORIGIN`, `RUSTDB_QUERY_TIMEOUT_SECONDS`, `RUSTDB_CONNECT_TIMEOUT_SECONDS`, `RUSTDB_MAX_ROWS`, `RUSTDB_STATIC_DIR`, `RUSTDB_BUILD_IMAGE`, `RUST_LOG`, `POSTGRES_USER`, `POSTGRES_PASSWORD` e `POSTGRES_DB`.

Integrações opt-in usam `RUSTDB_TEST_DATABASE_URL`/`RUSTDB_TEST_PASSWORD` para PostgreSQL; `RUSTDB_MYSQL_TEST_HOST`, `PORT`, `DATABASE`, `USERNAME`, `PASSWORD` e `TYPE` (`mysql` ou `mariadb`); e `RUSTDB_SQLSERVER_TEST_HOST`, `PORT`, `DATABASE`, `USERNAME` e `PASSWORD`.

Nunca registrar valores, ambiente completo ou URLs privadas. `.env` permanece ignorado e não deve ser aberto. Testes devem usar instâncias descartáveis, rede privada e credenciais efêmeras. O Compose de produção continua contendo apenas Orisema Data e seu PostgreSQL interno; bancos de destino não são adicionados à infraestrutura da aplicação.

Sizing e política formal de observabilidade: A confirmar.
