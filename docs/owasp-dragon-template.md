# OWASP Dragon Template

Use este template para documentar ameacas e contramedidas seguindo o fluxo sugerido pelo OWASP Dragon. Substitua os itens marcados com `<...>` durante cada iteracao.

## 1. Contexto do Sistema
- **Aplicacao/Servicos:** <nome do sistema / servicos>
- **Objetivo de negocio:** <resuma o valor entregue>
- **Escopo da iteracao:** <release, sprint, componente>
- **Stakeholders principais:** <produto, engenharia, seguranca, ops>

## 2. Arquitetura (Canvas Dragon)
Preencha os blocos principais do canvas.
- **Componentes internos:** <dominio, servicos, pipelines>
- **Front-door / Interfaces externas:** <HTTP APIs, filas, integracoes>
- **Dados sensiveis:** <PII, credenciais, segredos>
- **Dependencias externas:** <provedores, integracoes de terceiros>
- **Controles existentes:** <autenticacao, autorizacao, telemetria, infraestrutura>
- **Assumptions:** <premissas de confianca, limites de ambiente>

## 3. Atores e Motivacoes
| Ator | Motivacao | Acesso atual |
| --- | --- | --- |
| <Administrador> | <Objetivo legitimo ou malicioso> | <Rotas / permissoes> |
| <Atacante externo> | <Fraude, DoS, escalada> | <Pontos de entrada> |
| <Equipe operacao> | <Monitorar, manter> | <Ferramentas / privilegios> |

## 4. Fluxos Criticos
Liste os fluxos a serem analisados (auth, CRUD, pipelines, observabilidade). Para cada fluxo, vincule casos de teste BDD, logs e metricas associadas.
- <Fluxo 1>: requisitos, dados tocados, pre-condicoes, pos-condicoes, cenarios BDD relacionados.
- <Fluxo 2>

## 5. Identificacao de Ameacas
Utilize STRIDE/LINDDUN ou taxonomias equivalentes.
| Fluxo | Ameaca | Categoria | Impacto | Probabilidade | Evidencia |
| --- | --- | --- | --- | --- | --- |
| <Fluxo 1> | <SQLi / brute force> | <Spoofing/Repudiation/...> | <Alto/Medio/Baixo> | <Alta/Media/Baixa> | <Referencias a logs/testes> |

## 6. Controles e Lacunas
| Ameaca | Controle existente | Eficacia | Lacuna identificada | Plano/Owner |
| --- | --- | --- | --- | --- |
| <Threat X> | <JWT + rate limiting> | <Boa/Moderada> | <Ex.: sem MFA> | <Responsavel + prazo> |

## 7. Plano de Mitigacao
- **Rapido (<= sprint atual):** <tarefas acionaveis>
- **Medio prazo:** <iniciativas de hardening>
- **Longo prazo:** <capacidades estruturantes>

## 8. Evidencias e Testes
- **Cenarios BDD/automatizados:** <`tests/features/...`>
- **Alarmes / dashboards:** <Prometheus, Grafana>
- **Requisitos de auditoria:** <logs sanitizados, trilhas>
- **Checklist deployment:** <verificacoes antes do go-live>

## 9. Aprovacao & Acompanhamento
- **Data da revisao:** <dd/mm/aaaa>
- **Participantes:** <nomes e funcoes>
- **Decisoes:** <resumo>
- **Follow-up:** <link para Jira/ADO/GitHub>

Versione este documento junto com o codigo para manter o contexto do risco sempre atualizado.