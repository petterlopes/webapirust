# Visao Geral do Modelo de Ameacas

## Ativos
- Dados de contas de usuario no PostgreSQL (PII: nome, email, papel, timestamps).
- Credenciais sensiveis: hashes Argon2id, segredos JWT, senha bootstrap.
- Logs estruturados e eventos de auditoria contendo contexto operacional.
- Metricas de negocio e telemetria (Prometheus/Grafana).
- Pipelines/CI e scripts de deploy que carregam variaveis de ambiente.

## Atores e Pontos de Entrada
- **Administrador autenticado**: CRUD completo via `/users`; risco de abuso de privilegio e engenharia social.
- **Viewer autenticado**: acesso somente ao proprio perfil; pode tentar elevar privilegios ou abusar de bugs de autorizacao.
- **Atacante externo**: atinge `/auth/login`, `/health`, `/docs`, `/metrics` (quando exposto); pode fazer brute-force, fuzzing ou DoS.
- **Operacoes/CI/CD**: pipelines que aplicam migracoes, injetam segredos e publicam imagens.
- **Observabilidade**: operadores com acesso a Prometheus/Grafana e logs agregados.

## Fronteiras de Confianca
1. Internet publica  API Axum (`http://:8080`).
2. API  Banco PostgreSQL (rede interna).
3. API  Stack Prometheus/Grafana (rede de observabilidade).
4. Estacoes de desenvolvimento  repositorio de codigo e pipelines.

## Principais Ameacas e Mitigacoes
| Ameaca | Impacto | Mitigacoes |
| --- | --- | --- |
| Credential stuffing / brute force em `/auth/login` | Sequestro de contas admin ou viewer | Hash Argon2id, respostas uniformes, auditoria de tentativas, rate limiting global. **Pendente**: MFA e bloqueio progressivo. |
| Escalada de privilegio (viewer -> admin) | Alteracao nao autorizada de dados | Autorizacao centralizada em `UserService`, controllers nao expostos sem JWT, DTOs nao incluem campos proibidos. **Pendente**: revisitar escopos finos e alertas de acao privilegiada. |
| Violacao de invariantes do dominio | Dados inconsistentes no banco | Value objects (`EmailAddress`, `UserName`, `PlainPassword`) e `PasswordHash::new` impedem entrada invalida; repositorio converte registros usando `User::try_new`; cenarios BDD garantem autenticacao consistente. |
| Vazamento de PII em logs/auditoria | Exposicao de informacao sensivel | `sanitize_for_logging` remove caracteres de controle, limita tamanho, audit trail armazena apenas email sanitizado e ID. **Pendente**: mascarar partes do email e definir politica de retencao. |
| Corpos JSON excessivos | DoS por consumo de memoria | `RequestBodyLimitLayer` limitado a 16 KiB + rate limiting. |
| SQL Injection | Corrupcao / exfiltracao de dados | SQLx com parametros, sem concatenacao dinamica. |
| Comprometimento de segredos (JWT/banco/bootstrap) | Uso indevido de credenciais | Segredos via `.env`/variaveis, alerta para desativar bootstrap. **Pendente**: gerenciador central de segredos e rotacao automatica. |
| Trafego sem TLS | MitM, sniffing de credenciais | **Pendente**: TLS terminando em proxy confiavel, cabecalhos HSTS/STS e configuracao de cipher suite. |
| Falta de monitoracao de integridade | Ataques passam despercebidos | Metricas de operacao (`app_user_operations_total`), auditoria e dashboards Grafana. **Pendente**: alertas e integracao com SIEM. |
| Supply chain / dependencias desatualizadas | Risco de CVE | `Cargo.lock` versionado, revisao manual. **Pendente**: `cargo audit/deny`, verificacoes de imagem Docker (Trivy). |

## Suposicoes
- API fica atras de proxy reverso confiavel que aplica TLS e cabecalhos de seguranca (fora do escopo do repo).
- PostgreSQL roda em rede interna com autenticacao e backup configurados.
- Stack de observabilidade acessivel apenas ao time de operacoes.
- Pipelines e secrets managers protegem segredos e nao logam dados sensiveis.

## Itens em Aberto
- Implementar MFA ou bloqueio progressivo para administradores.
- Forcar HTTPS/TLS e adicionar security headers padrao (HSTS, CSP minima, referrer policy).
- Refinar autorizacao (RBAC/ABAC) e adicionar eventos de dominio para rastrear mudancas criticas.
- Automatizar scanners de dependencias e imagens (cargo audit/deny, Trivy/Grype) no CI.
- Endurecer imagens Docker (usuario nao-root, fs read-only, drop capabilities, network policies).
- Definir politica de retencao e alerta para logs/auditoria, integrando com SIEM.
- Criar plano de resposta a incidentes baseado nos dados de auditoria.
