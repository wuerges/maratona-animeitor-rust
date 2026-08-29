# Status do refactor multi-evento

Estado atual do refactor descrito em [multi-event.md](multi-event.md). Atualizado a cada commit do refactor.

## Feito

### Base (commit WIP `3ef8f99`, sessão anterior)

- **API interna completa** (`/internal`, HTTP Basic por token) conforme [event-api.md](event-api.md): CRUD de events/contests/sites, PATCH de tempo (negativo = countdown), runs incrementais com correção por id, rotação de salts nos três níveis, envelope `{ data, errors, warnings }`.
- **EventStore multi-evento** (`server/service/src/event_store.rs`): `HashMap<EventName, Event>`, broadcasters de runs (membroadcast com replay) e timer por evento, relay de remote_control por contest, derivação de chaves de site (HMAC-SHA256, base62, 12 chars).
- **API pública nova** conforme [public-api.md](public-api.md): `GET /api/events`, `.../contest`, `.../config`, `.../runs_secret` (chave via header Bearer, fora da URL), WS `.../runs_ws`, `.../timer`, `.../remote_control/{key}`, `/api/metrics`.
- **Cliente**: roteamento por caminho `/animeitor/{event}/{contest}/`, landing, SDK com URLs novas, chave do site via Bearer, mídia com formatos de URL do config do contest, bridge `legacy.rs` (client-model intocado).
- **Endpoints legados removidos** (decisão do usuário: pular a janela de compatibilidade da Fase 2 do plano).
- nginx api-only corrigido (prefixo `/api` preservado; `try_files` para o SPA).

### Continuação (2026-08-28)

- **Fixes da spec na API interna**: `answer` inválido responde 400 `invalid_value`; salts com corpo vazio/`{}`/`""` geram salt aleatório (event-api.md).
- **Endpoints de leitura internos** (usados pelo printurls): `GET /internal/events`, `GET /internal/events/{e}/contests`, `GET /internal/events/{e}/contests/{c}/sites` (salts inclusos — escopo interno).
- **`PublicConfig.sites`** omitido quando vazio.
- **403 `not_started`** antes do início (`time_seconds < 0`) nos endpoints públicos do contest (contest, config, runs_ws, runs_secret); lista de eventos e timer continuam disponíveis (decisão do usuário).
- **Feeder standalone** (`update_contest_state`): publica via `/internal`, preserva o salt do evento entre ticks e cria o contest padrão (codes `[""]`); o loop `-i` do simples foi removido junto com `--default-event` e `dbupdate_v2`.
- **printurls reescrito**: lê a API interna e imprime URLs de contest e reveleitor (chave derivada via HMAC + `?secret=`/`&sede=`); args antigos `-s/-x/-y` (sedes/secrets/salt) removidos do cli.
- **Cliente**: tela de countdown (nomes vindos do caminho), fix do flash na primeira pintura (placeholder negativo no timer), parsing de caminho movido para client-model (`path.rs`) com testes nativos.
- **Deploy**: Dockerfile com `--public-url /animeitor/` e binário do feeder na imagem; serviço `feeder` no compose; `.env` sem `SECRET`/`SEDES` e com `SERVER_URL`; Makefile com mount raiz (landing) e `run-standalone-loop` via feeder; naquadah.Makefile, docker-compose.regional-exemplo.yaml e server/README atualizados.
- **Changelog** com as entradas da migração; [multi-event.md](multi-event.md) aponta para este documento.

## Pendente (backlog)

- Testes de handshake de WebSocket (sem harness actix-ws nos testes).
- Fluxo S3 (`serve-as-bucket/`): conferir landing e `config.json` nesse modo (chaves de `config.json` conferem com `SdkConfig`; falta teste real).
- Flags antigas em `config/regional_*/Makefile` (deferido).
- Erros de envelope no client-sdk (`enveloped`) continuam com retry de 5s, agora logados como `error!`; sem estado de erro visível na UI.
- Defaults de mídia (`photos/{team_login}.webp` etc.) são aplicados no cliente, não no servidor.

## Decisões registradas

- Endpoints legados ficam removidos (sem janela de compatibilidade).
- `?secret=` ainda é aceito como atalho no cliente, mas a chamada à API usa header Bearer.
- Pre-start: endpoints públicos do contest respondem 403 `not_started`; remote_control WS continua aberto (não vaza estado).
- O contest padrão do feeder usa codes `[""]` (regex vazia casa com qualquer login; convenção do `config/basic.toml`).
