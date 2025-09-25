# WebRust Academy

Bem-vind@! Este guia continua sendo um apoio didatico, agora alinhado com o refino em torno de DDD, seguranca e observabilidade da API **WebRust**. A intencao e mostrar como cada camada trabalha, quais decisoes de arquitetura foram tomadas e como voce pode evoluir o projeto com confianca.

---

## 1. Visao geral do projeto

- **Objetivo**: API REST para gerenciar usuarios com [Axum](https://docs.rs/axum/latest/axum/), aplicando principios de Clean Architecture + DDD tatico.
- **Infraestrutura**: Postgres via SQLx, autenticacao Argon2id + JWT, observabilidade com Prometheus/Grafana, audit trail estruturado.
- **Diferenciais atuais**:
  - Value objects garantem invariantes (email, nome, senha) e padronizam validacao.
  - Service layer centraliza regras de autorizacao (admin x viewer) e sanitiza campos antes de auditar.
  - Documentacao OpenAPI automatica com utoipa e Swagger UI acoplada.
  - Limitacao de body (16 KiB), rate limiting e contadores de metricas por operacao.
  - BDD com cucumber cobrindo autenticacao, usando repositorio em memoria para cenarios repetiveis.

Use este projeto como laboratorio de boas praticas: isolamos dominio, tratamos telemetria desde o inicio e mantemos a seguranca na linha de frente.

---

## 2. Pre-requisitos essenciais

| Ferramenta | Versao | Observacoes |
|------------|--------|-------------|
| Rust toolchain | 1.90+ | `rustup toolchain install stable`
| SQLx CLI (opcional) | 0.7+ | `cargo install sqlx-cli --no-default-features --features native-tls,postgres`
| Docker Desktop ou Rancher Desktop | Atual | Compose para subir app + monitoramento
| NerdCTL (se usar Rancher) | 1.7+ | CLI para containerd (`nerdctl compose ...`)
| Insomnia/Postman | Atual | Colecao pronta em `insomnia.json`
| Prometheus + Grafana | Provisionados pelo `docker compose` |

---

## 3. Executando o projeto

### 3.1 Rodando apenas a API
```
cp .env.example .env
cargo run
```
- Aplica migracoes automaticamente (`sqlx::migrate!`).
- Sobe o servidor em `http://localhost:8080`.
- Rate limiting e metricas sao montados por padrao.

### 3.2 Stack completa com Docker Compose
```
docker compose up --build
```
Servicos provisionados:
- `app`: API Axum (porta 8080)
- `db`: PostgreSQL (porta 5432)
- `prometheus`: scraping `/metrics` (porta 9090)
- `grafana`: dashboards (porta 3000, login `admin/admin`)

> Ao alterar codigo, rode `docker compose build app --no-cache && docker compose up -d app` para recompilar o binario Rust sem reaplicar o restante da stack.

### 3.3 Rancher Desktop / nerdctl
1. Ajuste o contexto Docker:
   ```powershell
   docker context use rancher-desktop
   setx DOCKER_HOST npipe:////./pipe/rancher-desktop
   ```
2. Recarregue o terminal.
3. Use `docker compose up --build` ou `nerdctl compose up --build`.

---

## 4. Arquitetura em camadas (DDD tatico)

```
src/
  domain/
    entities/        -> `User`, `NewUser`, `UpdateUser`, `UserRole`
    value_objects/   -> `EmailAddress`, `UserName`, `PlainPassword`, `PasswordHash`
    errors.rs        -> `DomainError`
    repositories/    -> contratos (`UserRepository`)
  application/
    services/        -> `AuthService`, `UserService`
    dtos/            -> shapes expostos na API (CreateUserDto, etc.)
  infrastructure/
    repositories/    -> adaptadores SQLx para Postgres
    database.rs      -> criacao de pool
  presentation/http/
    controllers/     -> handlers Axum + anotacoes utoipa
    auth/            -> extractor `CurrentUser`, guards de papel
    routes.rs        -> registro de rotas
    docs.rs          -> definicao do OpenAPI
  shared/
    error.rs         -> `AppError`, conversao para HTTP
    validation.rs    -> sanitizacao e limites
  telemetry/
    logging.rs, metrics.rs, audit.rs
  app/
    router.rs, state.rs, rate limiter
```

- **Dominio** nao conhece HTTP, banco ou JWT. Tudo e oferecido em tipos fortes e invariantes.
- **Application** recebe DTOs, traduz para value objects, executa regras de negocio e fala com o repositorio via contrato.
- **Infrastructure** so mapeia I/O (SQLx -> Dominio). Qualquer dado fora do padrao vira `DomainError`.
- **Presentation** serializa/deserializa JSON, injeta autenticacao e chama os servicos.
- **Telemetry** orquestra logging estruturado, metricas, auditoria e rate limiting.

Esse formato facilita testes, substituicao de adaptadores e leitura do codigo.

### 4.1 Clean Architecture e SOLID na pratica
- **Separacao de responsabilidades**: value objects validam entrada, services orquestram regras e controllers apenas adaptam HTTP.
- **Dependencia apontando para dentro**: `presentation`, `infrastructure` e `telemetry` utilizam apenas contratos de `application`/`domain`, permitindo substituicoes (ex.: repositorio em memoria nos testes BDD).
- **Interfaces pequenas**: `UserRepository` oferece apenas as operacoes necessarias; implementacoes diferentes (Postgres, memoria) preservam o contrato.
- **Inversao de dependencia**: servicos recebem `Arc<dyn UserRepository>` e `JwtManager`, mantendo dominio independente de detalhes externos.


---

## 5. Tour por partes importantes do codigo

### 5.1 Value objects de seguranca
```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse<S: AsRef<str>>(value: S) -> Result<Self, DomainError> {
        // trim, valida regex, tamanho maximo etc.
    }
}
```

- `PlainPassword::parse` exige minimo de 12 chars, letra maiuscula, minuscula, digito e simbolo.
- `PasswordHash::new` impede armazenar hash vazio.
- Todos os handlers devem passar por esses tipos antes de persistir dados.

### 5.2 UserService
```rust
pub async fn create_user(
    &self,
    actor: &AuthenticatedUser,
    dto: CreateUserDto,
) -> AppResult<UserResponseDto> {
    ensure_admin(actor)?;
    self.create_user_internal(dto).await
}
```

- `ensure_admin` garante que apenas administradores criam/atualizam/removem usuarios.
- Conversoes de DTO -> value objects ocorrem no service (`UserName::parse`, `EmailAddress::parse`).
- Auditoria e metricas sao disparadas nos controllers, mantendo servicos puros.

### 5.3 AuthService
- Recupera usuario via email, valida senha com Argon2id, emite JWT via `JwtManager`.
- `AuthenticatedUser` guarda `Uuid`, `email`, `UserRole` com helpers (`id()`, `email()`, `role()`).
- `CurrentUser` extractor valida header `Authorization` e rejeita tokens invalidos antes de chegar nos handlers.

### 5.4 Router e middlewares
```rust
Router::new()
    .merge(routes::auth_routes())
    .merge(routes::user_routes())
    .route("/health", get(health_check))
    .route("/metrics", get(metrics_handler))
    .merge(swagger_ui)
    .layer(RequestBodyLimitLayer::new(MAX_JSON_BODY_BYTES))
    .layer(CorsLayer::permissive())
    .layer(rate_limiter_layer)
    .layer(metrics_layer)
    .layer(TraceLayer::new_for_http())
```
- Ordem das camadas importa: limite de body -> CORS -> rate limit -> metricas -> tracing.
- `metrics_handler` remove quebras de linha iniciais para Prometheus aceitar o payload.

### 5.5 Auditoria
- `telemetry::audit` define `AuditEvent::success` e `AuditEvent::failure`.
- Controllers chamam `state.audit().log(...)` com ator (id, email sanitizado, papel) e alvo (UUID do usuario tocado).
- Ideal para futuras integracoes com SIEM.

---

## 6. Observabilidade

### 6.1 Logging
- Configuracao em `telemetry/logging.rs`.
- JSON estruturado inclui campos `level`, `target`, `span`, `file`, `line`.
- Ajuste nivel via `telemetry.log_level` no YAML ou `RUST_LOG`.

### 6.2 Metricas
- `/metrics` exposto automaticamente; use Prometheus ou `curl`.
- Contadores principais: `app_user_operations_total` (por operacao e resultado), `app_user_operation_duration_seconds` (histograma).
- `metrics_layer` agrega `TraceLayer` com `tower_http` para padrao `axum_http_requests_total`.

### 6.3 Grafana
- Dashboard base em `monitoring/grafana/dashboards/json/webrust-overview.json`.
- Ajuste ou adicione paineis provisionando novos arquivos JSON.

### 6.4 Troubleshooting rapido
```
# Targets Prometheus
(Invoke-RestMethod -Uri http://localhost:9090/api/v1/targets).data.activeTargets \
  | Where-Object { $_.scrapePool -eq "webrust" }

# Logs estruturados
nerdctl compose logs app
```

---

## 7. Seguranca e conformidade

- Threat model atualizado em `docs/threat-model.md` lista ativos, atores, fronteiras, ameacas e mitigacoes pendentes.
- Referencia ASVS 5.0 em `docs/asvs/OWASP_Application_Security_Verification_Standard_5.0.0_en.pdf`.
- Controles implementados
  - Autenticacao com Argon2id + JWT (expiracao configuravel).
  - Autorizacao na camada de servico (admin vs viewer).
  - Sanitizacao antes de auditar.
  - Rate limiting e limite de payload.
  - Uso de SQL parametrizado via SQLx.
- Gap principais (priorize no roadmap): MFA/lockout, TLS obrigatorio, politicas mais granulares, escaneamento continuo de dependencias, endurecimento das imagens Docker.

---

## 8. Exercicios e futuras evolucoes

1. **Tests de dominio**: escreva testes unitarios para `EmailAddress`, `PlainPassword` e `UserService` cobrindo caminhos de erro.
2. **BDD**: amplie os arquivos em `tests/features` cobrindo viewer vs admin, erros de validacao e fluxos negativos.
3. **Eventos de dominio**: emita eventos quando usuarios forem criados/alterados/removidos e processe em `application`.
4. **Hardening**: habilite usuarios nao-root nos containers e monte filesystem somente leitura.
5. **Auditoria**: enviar eventos para fila externa (RabbitMQ/Kafka) para retencao prolongada.
6. **CI/CD**: pipeline com `cargo fmt`, `clippy`, `test`, `sqlx prepare`, `cargo deny`, build da imagem e publicacao.

---

## 9. Ferramentas de apoio

- **Threat model**: `docs/threat-model.md`
- **README operacional**: `README.md`
- **Colecao Insomnia**: `insomnia.json`
- **Dashboard Grafana**: `monitoring/grafana/dashboards/json/webrust-overview.json`
- **Template OWASP Dragon**: `docs/owasp-dragon-template.md`
- **Config padrao**: `configuration/default.yaml`

Continue experimentando: Rust recompensa disciplina, e o WebRust oferece um ambiente controlado para conectar teoria, boas praticas e operacao real.


