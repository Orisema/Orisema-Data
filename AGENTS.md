# Instruções para agentes de IA

Este arquivo rege qualquer agente de IA que trabalhe neste repositório. A pasta `.ai/` é o contexto persistente e versionado do projeto; ela documenta o estado confirmado, mas não substitui a leitura do código e da documentação de origem.

## Leitura obrigatória antes de modificar código

Antes de qualquer alteração em código, configuração, migration ou teste:

1. Leia integralmente `AGENTS.md`.
2. Leia integralmente os cinco arquivos de contexto: `.ai/PROJECT.md`, `.ai/ARCHITECTURE.md`, `.ai/DATABASE.md`, `.ai/DECISIONS.md` e `.ai/HANDOFF.md`.
3. Execute `git status --short` e identifique alterações existentes.
4. Preserve alterações alheias; não reverta, sobrescreva, formate em massa nem inclua no próprio trabalho arquivos fora do escopo.
5. Consulte README, documentação e fontes relevantes ao escopo. Não trate roadmap ou sugestão como funcionalidade existente.

## Regras de trabalho

- Documente somente fatos confirmados pelo repositório ou por evidência verificável. Quando não houver evidência, escreva exatamente `A confirmar`.
- Nunca leia, exponha, copie ou versione `.env`, credenciais, tokens, chaves, senhas ou connection strings privadas. `.env.example` pode ser consultado, mas seus placeholders não são credenciais reais.
- Não refatore arquitetura incidentalmente. Mantenha mudanças pequenas e explícitas.
- Antes de entregar, execute os testes relevantes ao escopo. Para alterações Rust, considere formatação, check, testes e Clippy; para frontend, lint, testes e build. Testes de integração exigem PostgreSQL descartável e variáveis próprias de teste.
- Registre em `.ai/DECISIONS.md` somente decisões arquiteturais ou operacionais claras, com evidência. Não registre hipóteses.
- Atualize `.ai/HANDOFF.md` ao final de cada trabalho para refletir apenas o estado atual, próximos passos e bloqueios; não o use como diário histórico.
- Atualize `.ai/TODO.md` quando o trabalho confirmar, sugerir ou deixar pendências relevantes, mantendo as categorias definidas no arquivo.
- Revise `git diff`, `git diff --check` e o diff em busca de segredos antes da entrega.

## Trabalho multiagente

- Cada agente deve declarar e respeitar um escopo de arquivos. Evite dois agentes editarem o mesmo arquivo simultaneamente.
- Antes e depois de editar, confira `git status --short` e preserve alterações de outros agentes e do usuário.
- Compartilhe dependências, contratos e bloqueios pelo contexto versionado, principalmente `.ai/HANDOFF.md`; consolide a atualização final para evitar conflitos.
- Use branches no formato `ai/<agente>/<tarefa>`, com nomes curtos em minúsculas e separados por hífen.
- Para trabalhos simultâneos, integre por Pull Requests separados direcionados à branch de integração acordada. Não faça commit, push, merge ou Pull Request sem autorização explícita.
- Um agente integrador deve revisar sobreposição de arquivos, migrations, contratos de API e resultados de testes antes da integração.

## Fontes de verdade

- Implementação: `crates/`, `frontend/src/`, `migrations/`.
- Build e execução: `Cargo.toml`, manifests dos crates, `frontend/package.json`, `Dockerfile`, `compose.yml`, `.gitlab-ci.yml` e `.env.example`.
- Uso e segurança: `README.md`, `SECURITY.md`, `CONTRIBUTING.md` e `docs/`.
- Contexto resumido para agentes: `.ai/`. Em divergência, valide no código e corrija o contexto.
