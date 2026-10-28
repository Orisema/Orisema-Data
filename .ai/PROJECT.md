# Projeto

## Identidade e estado

Orisema Data v0.4.0 é um cliente web de SQL e gerenciamento de bancos PostgreSQL, MySQL, MariaDB e SQL Server, escrito em Rust. O README o classifica como MVP experimental para uma estação local confiável, em desenvolvimento ativo. A v0.4 preserva autenticação e autorização e adiciona drivers locais de destino, mas ainda não é classificada como pronta para produção. A licença declarada pelo workspace e frontend é AGPL-3.0-only.

O repositório remoto documentado é GitLab. O fluxo de contribuição existente recomenda Conventional Commits e Merge Requests. A convenção adicional para agentes está em `AGENTS.md`.

## Capacidades confirmadas

- Criar, ler, atualizar, excluir e testar conexões PostgreSQL, MySQL, MariaDB e SQL Server.
- Criptografar senhas persistidas com AES-256-GCM e chave fornecida pelo ambiente.
- Explorar schemas, tabelas, views e colunas sob demanda.
- Executar uma instrução SQL por requisição, com política de somente leitura opcional, limites, timeouts, cancelamento e erros sanitizados.
- Editar SQL com Monaco, executar seleção, manter abas e rascunhos, histórico, favoritos e consultas salvas no navegador.
- Exibir resultados paginados no cliente, copiar valores e exportar CSV UTF-8.
- Servir o frontend pelo Vite em desenvolvimento ou pelo servidor Axum a partir de arquivos estáticos.

## Organização

- `crates/orisema-data-core`: contratos compartilhados e trait de driver.
- `crates/orisema-data-server`: API Axum, configuração, serviço, repositório, criptografia e drivers de destino.
- `frontend`: React 19, TypeScript, Vite, TanStack Query e Monaco.
- `migrations`: migrations SQLx embutidas para o banco interno.
- `docs`: arquitetura, API, roadmap, publicação e registros de verificação.
- `scripts/init-env.mjs`: gera configuração local aleatória e se recusa a sobrescrever `.env` existente.

## Limites confirmados

- A v0.4 preserva autenticação, usuários, papéis admin/editor/viewer, trilha de auditoria e políticas por conexão aplicadas no backend.
- Não há Agent remoto, acesso remoto ou sessões temporárias; os únicos drivers de destino são PostgreSQL, MySQL, MariaDB e SQL Server.
- A aplicação não deve ser exposta à Internet nem usada com contas privilegiadas ou de produção.
- Consultas persistidas no navegador não são criptografadas.
- O modo somente leitura não é sandbox para funções, extensões ou efeitos permitidos ao papel no banco de destino.

## Comandos de qualidade documentados

```sh
cargo fmt --all -- --check
cargo check --locked --workspace
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cd frontend
npm ci
npm run lint
npm test
npm run build
```

O teste Rust com PostgreSQL real é ignorado por padrão e requer banco descartável, `RUSTDB_TEST_DATABASE_URL`, `RUSTDB_TEST_PASSWORD` e `--include-ignored`.
