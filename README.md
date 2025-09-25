# WebRust

API REST em Rust com Axum 0.7 estruturada em DDD tatico, cobrindo autenticacao JWT, autorizacao por papeis, auditoria sanitizada, observabilidade com Prometheus/Grafana e integracao com utoipa para OpenAPI. O projeto esta pronto para execucao local ou via Docker Compose, com stack de monitoramento provisionada automaticamente.

## Destaques
- Dominio modelado com value objects (`EmailAddress`, `UserName`, `PlainPassword`, `PasswordHash`) que garantem invariantes antes de persistir dados.
- Camada de servicos centraliza regras de negocio e verificacao de autorizacao, mantendo os controllers finos.
- Auditoria estruturada (`target=audit`) registra quem executou cada acao; campos sao sanitizados antes de entrar em logs.
- Limite de corpo JSON (16 KiB), rate limiting e metricas expostas em `/metrics` via `axum-prometheus`.
- Documentacao OpenAPI gerada automaticamente com utoipa + Swagger UI em `/docs`.
- Colecao Insomnia atualizada com login e fluxo completo de usuarios (`insomnia.json`).
- Suite BDD com cucumber valida o fluxo de autenticacao end-to-end (`cargo test --test bdd`).

## Arquitetura DDD
```
src/
  domain/             # Entidades, value objects, erros de dominio e contratos
  application/        # DTOs e servicos (casos de uso) que orquestram o dominio
  infrastructure/     # Adaptadores externos (Postgres via SQLx, hashing, JWT)
  presentation/http/  # Controllers Axum, rotas, extratores e docs utoipa
  app/                # Bootstrap do estado compartilhado, router e rate limiter
  telemetry/          # Logging estruturado, metricas e auditoria
  shared/             # Tipos utilitarios (AppError, validacao)
configuration/        # Arquivos YAML de configuracao
monitoring/           # Prometheus + Grafana provisionados
migrations/           # SQL incremental aplicado via `sqlx::migrate!`
```
Os controllers delegam 100% da logica de negocio para os servicos de aplicacao. Repositorios convertem registros para entidades usando os value objects, impedindo que dados invalidos atravessem camadas.

## Clean Architecture e SOLID
- **Dependencias direcionadas ao dominio**: camadas externas (`presentation`, `infrastructure`, `telemetry`) dependem apenas de `application`/`domain`, garantindo inversao de dependencias.
- **Single Responsibility Principle**: value objects encapsulam validacao, services coordenam regras e cada controlador apenas traduz HTTP -> caso de uso.
- **Open/Closed Principle**: contratos (`UserRepository`, `PasswordHash`, `JwtManager`) permitem estender comportamentos sem alterar regras centrais.
- **Interface Segregation & Liskov**: contratos finos (`UserRepository`) evitam dependencias desnecessarias e repositorios alternativos (Postgres, in-memory) preservam semantica.
- **Dependency Inversion**: services trabalham com `Arc<dyn UserRepository>` e injetam `JwtManager`, mantendo dominio isolado de detalhes externos.


## Controles de seguranca implementados
- `POST /auth/login` com Argon2id e JWT (HS256) assinado com segredo configuravel.
- Checagem de papel na service layer: apenas `admin` acessa CRUD completo; `viewer` so enxerga os proprios dados.
- Logs de auditoria e metricas incluem duracao das operacoes e resultado (sucesso/erro).
- Sanitizacao de campos antes de logar (remove caracteres de controle e limita a 256 bytes).
- Rotas protegidas por extractor `CurrentUser` que valida e normaliza o token.
- Threat model mantido em `docs/threat-model.md`, alinhado ao OWASP ASVS 5.0.

Itens pendentes priorizados (ver threat model): MFA/lockout para administradores, TLS terminando no proxy, politicas de autorizacao mais granulares e pipeline de varredura de dependencias.

## Observabilidade e operacao
- Tracing estruturado JSON configurado em `telemetry/logging.rs` (ajuste via `telemetry.log_level` ou `RUST_LOG`).
- Metricas expostas em `/metrics`; o handler remove quebras de linha iniciais para compatibilidade com Prometheus.
- `/health` responde `"ok"` para probes.
- Layer de rate limit baseado em `tower_governor`, com contadores por operacao (`app_user_operations_total`).

## Autenticacao e autorizacao
- Credenciais bootstrap: `admin@webrust.dev` / `ChangeMe123!`. Mude apos o primeiro login e defina `APP__BOOTSTRAP__ENABLED=false`.
- Todas as rotas sob `/users` exigem header `Authorization: Bearer <token>`.
- Papeis suportados: `admin`, `viewer`. O JWT inclui `role`, verificado durante a autorizacao.
- Insomnia: execute a requisicao "Auth / Login" para preencher `{{ bearer_token }}` automaticamente.

## Documentacao da API
- Swagger UI: http://localhost:8080/docs
- OpenAPI JSON: http://localhost:8080/docs/openapi.json
- Componentes gerados por utoipa; qualquer alteracao em DTOs ou controllers requer recompilacao para refletir no schema.
- Erros padronizados em `crate::shared::error::ErrorResponse`, referenciados no OpenAPI.

## Configuracao
Ordem de precedencia: `configuration/default.yaml` < `configuration/local.yaml` (opcional) < variaveis `APP__*` < `.env`.

Chaves importantes:
- `server.host`, `server.port`
- `database.uri`, `database.max_connections`
- `jwt.secret`, `jwt.expires_in_seconds`
- `telemetry.service_name`, `telemetry.log_level`
- `rate_limit.requests_per_second`, `rate_limit.burst_capacity`

Exemplo (`.env`):
```
APP__SERVER__PORT=8080
APP__DATABASE__URI=postgres://postgres:postgres@localhost:5432/webrust
APP__JWT__SECRET=change_me_please
APP__JWT__EXPIRES_IN_SECONDS=3600
APP__RATE_LIMIT__REQUESTS_PER_SECOND=10
APP__RATE_LIMIT__BURST_CAPACITY=20
DATABASE_URL=postgres://postgres:postgres@localhost:5432/webrust
```

## Execucao local (sem containers)
```
cp .env.example .env
cargo run
```
O comando aplica migracoes (`sqlx::migrate!`), sobe o Axum em `http://localhost:8080` e ativa o rate limit/metricas automaticamente. Opcional: `docker compose up -d db` para iniciar apenas o PostgreSQL.

## Execucao com Docker Desktop
```
docker compose up --build
```
Servicos iniciados:
- `app` (porta 8080)
- `db` (PostgreSQL, porta 5432)
- `prometheus` (porta 9090)
- `grafana` (porta 3000)

Sempre que atualizar o codigo da API, rode `docker compose build app --no-cache && docker compose up -d app` para reconstruir o binario.

## Execucao com Rancher Desktop / nerdctl
1. Aponte o contexto Docker:
   ```powershell
   docker context use rancher-desktop
   setx DOCKER_HOST npipe:////./pipe/rancher-desktop
   ```
2. Reabra o terminal ou execute `refreshenv`.
3. Suba a stack com `docker compose up --build` ou `nerdctl compose up --build`.

## Prometheus e Grafana
- Prometheus UI: http://localhost:9090
- Grafana UI: http://localhost:3000 (login `admin/admin`)
- Datasource e dashboard "WebRust Overview" sao provisionados automaticamente.

Verificacoes uteis:
```powershell
# Targets Prometheus
(Invoke-RestMethod -Uri http://localhost:9090/api/v1/targets).data.activeTargets \
  | Where-Object { $_.scrapePool -eq "webrust" } \
  | Select-Object scrapeUrl, health, lastError

# Validar metrics dentro do container do Prometheus
nerdctl compose exec prometheus /bin/sh -c "wget -qO- http://app:8080/metrics | head"

# Validar metrics dentro do container da API
nerdctl compose exec app /bin/sh -c "curl -s http://localhost:8080/metrics | head"
```
Se Prometheus ou Grafana nao iniciarem apos atualizacoes, rode `docker compose up -d prometheus grafana` e confira `docker compose logs <servico>`.

## Banco de dados e migracoes
- Migracao inicial: `migrations/20250923165000_create_users_table.sql`.
- Migracoes sobem automaticamente em `cargo run` e no container (`sqlx::migrate!`).
- Execucao manual: `cargo sqlx migrate run` (requer `cargo install sqlx-cli --no-default-features --features native-tls,postgres`).

## Testes e qualidade
Sugestao de pipeline local:
```
cargo fmt
cargo clippy --all-targets --all-features
cargo test
cargo check --future-incompat-report
```
Para validar as invariantes de dominio, adicione testes unitarios em `src/domain/value_objects.rs` e `src/application/services`. Os cenarios BDD ja estao versionados em `tests/features` (execute `cargo test --test bdd`) e podem ser estendidos com novos fluxos (viewer vs admin, erros de validacao, etc.).

## Troubleshooting rapido
- **JWT invalido**: confirme `APP__JWT__SECRET` igual em todos os processos.
- **Prometheus sem dados**: verifique firewall local e se `app:8080/metrics` responde sem erro.
- **Erro de conexao SQLx**: confirme `DATABASE_URL` e que o banco aceitou conexoes externas.
- **Rate limit disparando**: ajuste `APP__RATE_LIMIT__BURST_CAPACITY` durante testes de carga.

## Roadmap recomendado
1. Implementar MFA ou lockout progressivo para administradores.
2. Forcar HTTPS/TLS no proxy frontal e adicionar security headers padrao.
3. Evoluir autorizacao para regras mais granulares (RBAC/ABAC) e eventos de dominio.
4. Automatizar varredura de dependencias (cargo audit/deny) e imagens Docker (Trivy ou Grype).
5. Escrever testes BDD cobrindo login, fluxo CRUD e cenarios de erro.
6. Integrar pipeline CI/CD com fmt/clippy/test/scan e publicacao da imagem.

Recursos adicionais:
- Threat model: `docs/threat-model.md`
- Analise ASVS: consulte `docs/asvs/OWASP_Application_Security_Verification_Standard_5.0.0_en.pdf`
- Guia educativo: `docs/index.md`
- Template OWASP Dragon: `docs/owasp-dragon-template.md`
- Colecao Insomnia atualizada: `insomnia.json`

---

Pronto! Com esse conjunto voce consegue rodar, observar e evoluir a API mantendo separacao de responsabilidades, seguranca e monitoracao desde o inicio do ciclo de desenvolvimento.
