# Handoff atual

## Estado

- Branch: `ai/codex/orisema-data-rust`, criada sobre o mesmo commit de `main` e preservando o trabalho v0.3/v0.4 que já estava no checkout sem commit.
- Os crates agora são `orisema-data-core` e `orisema-data-server`; o import Rust é `orisema_data_core`, o pacote frontend é `orisema-data-web` e o executável final é `orisema-data-server`.
- A identidade visível, Dockerfile, Compose, GitLab CI, README e documentação técnica usam Orisema Data.
- O histórico e as três branches anunciadas foram espelhados em `Orisema/Orisema-Data`; GitLab não anunciava tags. A comparação de refs após a publicação foi idêntica.

## Compatibilidade preservada

- Variáveis públicas e de teste `RUSTDB_*` permanecem inalteradas e são a única interface de ambiente implementada.
- Migrations 0001–0004, tabelas, endpoints, DTOs, criptografia AES-256-GCM e ciphertext `v1:` não foram alterados pela renomeação.
- O cookie `rustdb_session`, os defaults do banco interno e o volume persistente `rustdb-data` permanecem estáveis.
- O bind padrão continua em `127.0.0.1`; Compose publica o serviço somente em loopback.

## Arquitetura e segurança

- Core continua contendo o contrato/trait de driver; Server continua contendo HTTP, autenticação, autorização, serviços, persistência, criptografia e drivers concretos; Web continua independente como cliente React.
- A v0.4 já possui autenticação, papéis admin/editor/viewer, políticas e auditoria no backend. `docs/auth-rbac.md` propõe a evolução sem implementar funcionalidades extensas durante a renomeação.
- Não foi adicionado Agent, execução remota, acesso público ou dependência obrigatória de Orisema Infrastructure.

## Validação desta renomeação

- `git diff --check`: passou.
- `cargo fmt --all -- --check`: não executado; `cargo` não está instalado no ambiente.
- `cargo check --locked --workspace`: não executado pelo mesmo motivo.
- `cargo test --locked --workspace`: não executado pelo mesmo motivo.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: não executado pelo mesmo motivo.
- `npm ci`, `npm run lint`, `npm test` e `npm run build`: não executados; Node.js/npm não estão instalados no ambiente.
- Dockerfile e Compose: não executados; Docker/Podman não estão instalados no ambiente.
- As validações v0.4 anteriores registradas em `docs/verification.md` antecedem a renomeação e não substituem a bateria pendente acima.

## Riscos e próximos passos

- A branch reúne o trabalho v0.3/v0.4 preexistente e a renomeação porque esse trabalho estava sem commit no checkout; a revisão deve considerar o conjunto completo.
- Executar a bateria obrigatória em CI ou em host com Rust 1.92, Node.js 24 e runtime de contêiner antes do merge.
- Confirmar que instalações operacionais atualizem o nome do serviço Compose de `rustdb` para `orisema-data`; o volume e os dados não mudam.
- Definir proteção de `main`, checks obrigatórios e o estado final do GitLab; mantê-lo como referência durante a transição.
