# Pendências

## Confirmado

- Preservar PostgreSQL como banco interno e a criptografia existente das credenciais.
- Estender explicitamente o contrato de metadados antes de expor chaves e índices; o contrato v0.4 cobre schemas, tabelas/views e colunas/tipos.
- Adicionar testes automatizados de interface para login, usuários, auditoria, políticas e seleção/SQL gerado por driver.
- Definir e testar recuperação quando nenhum administrador ativo estiver disponível.
- Executar a bateria Rust, frontend e container da renomeação Orisema Data em ambiente com Cargo, Node/npm e Docker/Podman disponíveis.

## Sugestão

- Adicionar cancelamento nativo por identificador de sessão quando cada driver oferecer garantias melhores que a queda de socket.
- Adicionar CA personalizada sem introduzir material de certificado no banco ou nos logs.
- Tornar o limitador de login distribuído antes de múltiplas réplicas e planejar MFA/IdP.
- Definir retenção de auditoria, backup/restauração e rotação da chave.

## A confirmar

- Semântica desejada para DML `OUTPUT` do SQL Server no resultado comum.
- Matriz oficial de versões mínimas de MySQL, MariaDB e SQL Server.
- Política de expiração e limite de sessões; retenção obrigatória da auditoria.
- Após autorização, criar `Orisema/Orisema-Data`, espelhar branches/tags e definir o estado final do GitLab.
