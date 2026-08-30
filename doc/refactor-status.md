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
- **Deploy**: Dockerfile com `--public-url /animeitor/` e binário do feeder na imagem; serviço `feeder` no compose; `.env` sem `SECRET`/`SEDES` e com `SERVER_URL`; Makefile com mount raiz (landing) e `run-standalone-loop` via feeder; naquadah.Makefile, docker-compose.regional-exemplo.yaml, README raiz e server/README atualizados.
- **Changelog** com as entradas da migração; [multi-event.md](multi-event.md) aponta para este documento.

### Migração para axum (2026-08-29)

- **Server HTTP layer migrado de actix-web para axum 0.8** (TLS via axum-server 0.8): rotas, envelope, mensagens de WS e args do `simples` inalterados; dual-listen preservado (HTTP em `-p` + HTTPS em `--tls-port`, graceful shutdown coordenado).
- **Contest padrão `""` removido**: contests exigem nome não-vazio (400 `invalid_value`); feeder cria o contest `default`; novo endpoint público `GET /api/events/{event}/contests` (403 `not_started` pré-start) alimenta a landing, que agora lista contests por evento (`/animeitor/{event}/` sem contest não é mais caminho válido no cliente).
- **OpenSSL removido do workspace**: reqwest com rustls default; feature `vendored` eliminada do Dockerfile/Makefiles.
- Testes portados para `tower::oneshot` sobre o `Router` (20 testes: 13 internal + 7 public); smoke manual incluiu TLS dual-listen, SPA fallback, WS handshake 101 e printurls com chave derivada.

### Merge com regional2026/preparation (2026-08-30)

O merge do branch `regional2026/preparation` (hotfixes do contest 2026, realizado com sucesso em 2026-08-29) foi resolvido a favor do refactor; os hotfixes de servidor que seguem relevantes foram portados para o servidor axum:

- **Conflitos resolvidos**: `server/Cargo.toml`/`server/Cargo.lock` (workspace raiz vence — apagados), `server/server-v2/src/api.rs` (substituído por `public.rs`/`internal.rs` — apagado), `docker-compose.yaml` (modelo feeder + token), Makefile regional (sem env vars de build do trunk), `dataio.rs` (estrutura do refactor), manifests axum.
- **Fix de ordem do MOJ portado**: `read_runs` detecta direção do arquivo (ascending/descending) em vez do `.rev()` cego, com os testes de regressão `test_orders_stable_across_appends_{ascending,descending}`.
- **Client assets em memória portados de actix para axum** (`memory_files.rs`): pré-compressão gzip/brotli, ETag com sufixo de encoding, `Cache-Control` imutável para assets com hash, 304 em revalidação; mounts raiz (landing) e `/animeitor` (SPA fallback) servem da memória com uma carga única por pasta (canonicalizada); mídia (`photos`/`sounds`) continua em `ServeDir`. Testes unitários (negotiate/etag/hash) + integração (`tower::oneshot`).
- **Detecção de conexões WS mortas portada**: `runs_ws` e `timer_ws` leem a metade de leitura do socket (`tokio::select!` com `receiver.next()`), liberando FDs com o relógio congelado.
- **Compressão gzip das respostas da API** via `tower-http` `CompressionLayer` (escopo `/api` + `/internal`; assets estáticos ficam de fora por já saírem pré-comprimidos).
- **Makefile `config/regional_2026/` modernizado** (modelo do `naquadah.Makefile`): feeder `update_contest_state`, args `-v`/`-t`, `printurls --server/--token`, `cargo build -p server-v2`; criado `config/regional_2026/config.json` (prefixos de mídia estáticos). `client-v2/bucket` regenerado a partir do código mesclado.

## Pendente (backlog)

- Testes de handshake de WebSocket: com axum, o gating pré-upgrade dos WS (404/403 do `runs_ws`/`timer`/`remote_control`) é testável com requests HTTP puros (`oneshot`); falta adicionar esses testes. Fluxo completo de mensagens exigiria um client `tokio-tungstenite` contra um servidor spawnado.
- Fluxo S3 (`serve-as-bucket/`): conferir landing e `config.json` nesse modo (chaves de `config.json` conferem com `SdkConfig`; falta teste real).
- Flags antigas nos Makefiles de `config/regional_2024/` e `config/regional_2025/` (regional_2026 já migrado).
- Erros de envelope no client-sdk (`enveloped`) continuam com retry de 5s, agora logados como `error!`; sem estado de erro visível na UI.
- Defaults de mídia (`photos/{team_login}.webp` etc.) são aplicados no cliente, não no servidor.

## Decisões registradas

- Endpoints legados ficam removidos (sem janela de compatibilidade).
- `?secret=` ainda é aceito como atalho no cliente, mas a chamada à API usa header Bearer.
- Pre-start: endpoints públicos do contest respondem 403 `not_started`; remote_control WS continua aberto (não vaza estado).
- O contest padrão do feeder usa codes `[""]` (regex vazia casa com qualquer login; convenção do `config/basic.toml`).
