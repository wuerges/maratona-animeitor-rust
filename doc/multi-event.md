# Plano: múltiplos eventos em um único servidor animeitor

Este documento planeja a evolução do animeitor de **um evento por processo** para **vários eventos em um único servidor**, com a UI servida em `/animeitor/{event-name}/{contest-name}`, e a migração de todos os endpoints públicos para o escopo público `/api`. O escopo interno já está especificado em [event-api.md](event-api.md).

## Objetivos

- Um único servidor animeitor hospeda N eventos, criados e atualizados pela API interna (`/internal`).
- A UI fica endereçável por caminho: `/animeitor/{event-name}/{contest-name}/`.
- A API pública fica sob `/api`, espelhando a hierarquia interna (`events → contests → sites`).
- Um único build do cliente serve todos os eventos; o cliente descobre event/contest pelo próprio caminho.

## Layout de URLs

Três escopos convivem no mesmo servidor:

| Escopo | Caminho | Público? |
| --- | --- | --- |
| UI | `/animeitor/{event-name}/{contest-name}/` | sim |
| API pública | `/api/events/{event-name}/contests/{contest-name}/...` | sim |
| API interna | `/internal/events/{event-name}/contests/{contest-name}/...` | não (HTTP Basic) |

Convenções (idênticas às da API interna):

- O contest de nome `""` é o contest padrão, com segmento vazio no caminho: na UI, `/animeitor/{event-name}/` (barra final); na API, `/api/events/{event-name}/contests//...`.
- `/` e `/animeitor/` mostram uma landing page com a lista de eventos (seção Landing).
- `{event-name}` e `{contest-name}` são slugs (segmento de URL sem `/`).

## Servindo o cliente em `/animeitor/{event}/{contest}`

Hoje o build do cliente é servido por volumes estáticos do próprio servidor (`actix_files::Files` com `index.html`, `server/server-v2/src/lib.rs:20-27`), montado na raiz (`docker-compose.yaml`: `-v /dist:`) ou pelo fluxo S3 (`serve-as-bucket/`). Não há roteamento por caminho no cliente: a única rota é `path!("")` e o contest vem de `?contest=` (`client-v2/src/views/sedes.rs:218-227`, `client-v2/src/api.rs:9-25`).

Proposta:

1. **Um único build** com `trunk build --public-url /animeitor/` (o `trunk` injeta `<base href="/animeitor/">`), publicado como `/animeitor/index.html` + assets.
2. **SPA por caminho**: o servidor serve o mesmo `index.html` para qualquer `/animeitor/*`:
   - nginx (recomendado): `location /animeitor/ { try_files $uri /animeitor/index.html; }` — os dois modos atuais (`doc/nginx-examples/`) precisam ganhar essa regra.
   - actix (sem nginx): um handler fallback para `/animeitor/{event}/{contest}` que responde o `index.html` do volume, em vez de depender só de montagens estáticas fixas.
3. **Descoberta de event/contest**: o cliente lê `window.location.pathname`, extrai `{event-name}` e `{contest-name}` (vazio = contest padrão) e passa a usar caminhos na API em vez de `?contest=`.
4. **`config.json` global**: um único arquivo ao lado do `index.html` (`/animeitor/config.json`), buscado de forma relativa ao `<base>`, como hoje (`client-sdk/src/config.rs:135-147`). Continua sendo configuração de deploy (prefixos de API, fotos e sons); dados por contest vêm da API pública (`GET .../config`).
5. **Landing**: o mesmo build renderiza a lista de eventos quando o caminho é `/` ou `/animeitor/`, usando um novo `GET /api/events`; cada item aponta para `/animeitor/{event}/`.

## Revisão dos endpoints públicos atuais

Todas as rotas públicas de hoje ficam sob `web::scope("api")` (`server/server-v2/src/lib.rs:45-50`), com `Cors::permissive()` global (`lib.rs:43`). Seleção de contest via query `?contest=` (padrão `""`, `server/server-v2/src/api.rs:10-13`).

| Endpoint atual | Handler | O que serve | Classificação |
| --- | --- | --- | --- |
| `GET /api/contest?contest=` | `api.rs:59-75` | `ContestFile` (`server/data/src/lib.rs:114-129`) filtrado ao `titulo` do contest; `403` se `time < 0`, `404` contest desconhecido (`server/service/src/app_data.rs:71-81`) | leitura pública |
| `GET /api/config?contest=` | `api.rs:77-90` | `ConfigContest` (sedes, estilos, medalhas); mesmos `403`/`404` (`app_data.rs:84-94`) | leitura pública |
| `WS /api/allruns_ws?contest=` | `api.rs:115-153` | stream de `RunTuple`, com replay desde o início do servidor (`membroadcast`) e filtro pelo `titulo`; `403` contest desconhecido | leitura pública |
| `GET /api/allruns_secret?secret=&contest=` | `api.rs:92-113` | `RunsFile` não congelado da sede desbloqueada pelo secret (`app_data.rs:97-118`); `403` secret inválido ou `time < 0` | leitura com chave |
| `WS /api/timer` | `api.rs:155-189` | `TimerData { current_time, score_freeze_time }`, sem contest, pulando duplicatas | leitura pública |
| `GET /api/metrics` | `server/server-v2/src/metrics.rs:8-18` | métricas Prometheus (autometrics) | ops |
| `WS /api/remote_control/{key}` | `server/server-v2/src/remote_control.rs:15-117` | relay broadcast de `ControlMessage` (`WindowScroll`, `QueryString`, `PhotoState`) por chave | controle por chave |
| `PUT /api/contests` | `api.rs:191-220` | `ContestState` (runs + tempo + contest) com header `apikey` (chave `-k`, `server/server-v2/src/main.rs:34-36`); `201`/`401`/`500` | alimentador (feeder) |

Observações da revisão:

- O servidor hoje é **um processo, um banco**: um único `DB` compartilhado (`server/service/src/app_data.rs:23-31`, `server/service/src/dataio.rs:122-128`), alimentado por **um** feed (polling BOCA via `-i`, ou `PUT /api/contests`); o mapa de configs só filtra o feed único por `?contest=`. Não há conceito de evento no código.
- `PUT /api/contests` é um endpoint de **escrita pública** (autenticado por header) no escopo público — na proposta ele migra para a API interna (`POST /internal/events/{event}/runs`, etc.).
- `?secret=` na URL vaza a chave da sede em logs de acesso (nginx/actix) — na migração a chave sai da query.
- O `ConfigContest` público não contém secrets hoje; com a API interna, atenção para **nunca** expor `salt` nem chaves derivadas por `/api`.
- O exemplo nginx "api-only" (`doc/nginx-examples/api-only-server.conf`) faz `proxy_pass http://animeitor:8000/` com barra final, **removendo** o prefixo `/api` — incompatível com o escopo `api` atual do servidor (a requisição chegaria em `/contest`). Precisa de correção junto com a migração.

## Migração para `/api`

A API pública espelha a hierarquia interna. Endpoint a endpoint:

| Hoje | Proposto | Observações |
| --- | --- | --- |
| `GET /api/contest?contest=` | `GET /api/events/{event}/contests/{contest}/contest` | estado público do contest |
| `GET /api/config?contest=` | `GET /api/events/{event}/contests/{contest}/config` | config do contest (sedes, estilos, medalhas) |
| `WS /api/allruns_ws?contest=` | `WS /api/events/{event}/contests/{contest}/runs_ws` | stream público de runs |
| `GET /api/allruns_secret?secret=&contest=` | `GET /api/events/{event}/contests/{contest}/runs_secret` | chave do site via header `Authorization: Bearer <site-key>` (ver abaixo); o site é identificado comparando a chave com as chaves derivadas dos sites do contest |
| `WS /api/timer` | `WS /api/events/{event}/timer` | timer passa a ser por evento |
| `WS /api/remote_control/{key}` | `WS /api/events/{event}/contests/{contest}/remote_control/{key}` | relay por contest |
| `GET /api/metrics` | `GET /api/metrics` (inalterado) | global (ops) |
| `PUT /api/contests` | substituído por `POST /internal/events/{event}/runs` (e CRUD do evento) | o alimentador vira interno, com HTTP Basic por token ([event-api.md](event-api.md)) |
| — | `GET /api/events` | novo: lista de eventos para a landing |

Regras de segurança da migração:

- **Chave do site**: fora da query (vaza em logs); via header `Authorization: Bearer <site-key>`. A chave é a derivada dos 3 salts (`doc/event-api.md`, seção Salts); o servidor testa a chave recebida contra as chaves derivadas dos sites do contest para identificar o site. Chave inválida → `403`.
- **Nada sensível no escopo público**: `salt` de evento/contest/site e chaves derivadas nunca aparecem em respostas de `/api`.
- Mantém-se a porta do congelamento: `time < 0` → `403` nas leituras (`app_data.rs:71-81`).
- `Cors::permissive()` e `/api/metrics` aberto ficam como estão (reavaliar depois); o header `apikey` do `PUT /contests` desaparece junto com o endpoint.
- Contest padrão (`""`) usa segmento vazio, como na API interna.

## Arquitetura multi-evento do servidor

- O estado passa de um `DB` global para um mapa **por evento**: `HashMap<EventName, EventData>`, onde `EventData` contém o banco compartilhado (runs, contest, timer), os broadcasters (`runs_tx`, `time_tx`), os contests/sites/salts e um relay de remote_control **por contest**. O `AppData` atual (`service/src/app_data.rs:23-31`) é o ponto de partida natural dessa refatoração.
- Eventos são criados, atualizados e removidos **pela API interna** (`POST/DELETE /internal/events/{event}`, etc.). O servidor nasce vazio; eventos aparecem e somem em runtime.
- Event/contest desconhecidos na API pública → `404` (hoje é `403`/`404` dependendo do endpoint — unificar em `404` para recursos inexistentes, mantendo `403` para a porta do congelamento e chave inválida).
- O feed por evento substitui o modelo atual de um `DB` por processo: o polling BOCA (`-i`) e o `PUT /api/contests` migram para o alimentador interno (ou ficam como adaptadores que publicam via `/internal`, durante a transição).
- Configs CLI (`-s` sedes, `-x` secrets, `-y` salt, `cli`) deixam de definir o que é servido: contests, sites e salts passam a vir da API interna; os args continuam apenas para compatibilidade durante a migração.

## Mudanças no cliente

- **SDK** (`client-sdk/src/lib.rs:18-82`): os construtores de URL passam a montar `/api/events/{event}/contests/{contest}/<endpoint>` a partir de `SdkConfig.api_prefix` (padrão `/api` já existe, `config.rs:81-99`) + os dois segmentos; `create_secret_runs` usa o novo caminho e manda a chave por header. `ContestQuery` deixa de existir.
- **Roteamento** (`client-v2`): substitui `?contest=` pela leitura de `window.location.pathname` (`/animeitor/{event}/{contest}`); a única rota `path!("")` continua bastando, mas o estado inicial passa a vir do caminho. `sede`, `settings`, `secret` e `remote_control` seguem como query params.
- **Landing**: com caminho `/` ou `/animeitor/`, o app consulta `GET /api/events` e lista os eventos.
- **Reveleitor**: usa o novo `runs_secret` com a chave do site vinda das configurações (o `?secret=` da URL pode ser mantido como atalho, mas a chamada à API usa o header).
- **`config.json`**: permanece global por deploy (junto ao `<base>`), sem campo de evento; configuração por contest vem da API (`GET .../config`).

## Plano de migração

1. **Fase 1 — API interna**: implementar `/internal` conforme [event-api.md](event-api.md); o alimentador (`PUT /api/contests` e o loop BOCA) passa a publicar via API interna. `PUT /api/contests` continua aceito, mapeado para um evento padrão configurado.
2. **Fase 2 — servidor multi-evento**: `HashMap<EventName, EventData>`; novos caminhos públicos `/api/events/...`; endpoints legados (`/api/contest`, `/api/allruns_ws`, ...) mantidos por uma release, mapeados para o **evento padrão** configurado (`?contest=` vira o contest dentro desse evento). Corrigir o exemplo nginx api-only (preservar o prefixo `/api` no `proxy_pass`).
3. **Fase 3 — cliente**: build com `--public-url /animeitor/`, roteamento por caminho, landing, novas URLs da API; nginx/actix servem o SPA em `/animeitor/*` (`try_files`).
4. **Fase 4 — limpeza**: remover os endpoints legados, `?contest=`, `-s/-x/-y` e o feed BOCA/`PUT /contests` do escopo público.

Compatibilidade: durante as fases 2–3, `?contest=` continua funcionando nos endpoints legados (mapeado ao evento padrão); clientes antigos seguem operando até a Fase 4.

## Fora de escopo / em aberto

- Fotos e sons continuam em volumes globais (`/photos`, `/sounds`) — mídia por evento fica para depois.
- Fluxo de publicação S3 (`serve-as-bucket/`): o indexamento de qualquer caminho já funciona (IndexDocument), mas a landing e o `config.json` precisam de conferência nesse modo.
- TLS (`--tls-cert`/`--tls-key`) não muda.
- Builds de cliente por evento não são previstos: um build único.
- `client-model` não muda além do que o roteamento exigir.
