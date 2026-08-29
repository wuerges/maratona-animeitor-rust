# API pública

Esta API pública fica sob o escopo `/api` e espelha a hierarquia da API interna ([event-api.md](event-api.md)): `events → contests → sites`. Ela descreve os endpoints **como ficam após a migração** planejada em [multi-event.md](multi-event.md). Todos os tempos são expressos em **segundos**, sem exceção, e a unidade faz parte do nome do campo (ex.: `score_freeze_time_seconds`).

Salvo indicação contrária, os endpoints públicos **não exigem autenticação**. A única exceção é `runs_secret`, que exige a chave do site.

## Envelope de resposta

Como na API interna, toda resposta com corpo JSON é um objeto com os campos `data`, `errors` e `warnings`. Os três campos são **opcionais** e ausentes quando vazios:

- `data`: recurso ou resultado da operação; presente apenas em respostas de sucesso (2xx).
- `errors`: lista de objetos `{ "code": <string>, "message": <string> }`; presente apenas em respostas de erro (4xx/5xx).
- `warnings`: lista de objetos `{ "code": <string>, "message": <string> }`; problemas não fatais, presentes somente junto com `data`. Nenhum código de warning é definido por enquanto.

Handshakes de WebSocket não usam o envelope: a resposta do handshake é apenas o status HTTP (`101` em caso de sucesso), e as mensagens trafegadas são payloads nus (ver cada endpoint).

Códigos de erro desta API:

| code | status | situação |
| --- | --- | --- |
| `invalid_key` | 403 | chave do site ausente ou inválida |
| `not_started` | 403 | o evento ainda não começou |
| `not_found` | 404 | evento, contest ou site inexistente |

## Convenções

- `{event-name}`, `{contest-name}` e `{site-name}` são nomes de recursos; o contest de nome `""` é o contest padrão, com **segmento vazio** no caminho (ex.: `GET /api/events/{event-name}/contests//contest`).
- A API pública **nunca** expõe `salt` nem chaves derivadas.
- Antes do início (`time_seconds < 0`), nenhuma informação do contest é servida: os endpoints do contest respondem `403` com o código `not_started`. Ficam disponíveis apenas a lista de eventos, o timer do evento e o relay de controle remoto.

## Endpoints

### Listar eventos

- `GET /api/events`
- Lista os nomes dos eventos ativos, na ordem de criação. Usada pela landing (`/` e `/animeitor/`).

Resposta:

- `200 OK` — `data`: lista de strings com os nomes dos eventos.

Exemplo:

```json
{
    "data": ["regional-2026", "nacional-2026"]
}
```

### Estado público do contest

- `GET /api/events/{event-name}/contests/{contest-name}/contest`
- Estado público do contest: times e tempos. Os times são somente aqueles cujo login casa com `codes` do contest. Não contém `salt` nem chaves. Antes do início, o endpoint responde `403 not_started` (nenhuma informação do contest é servida).

Resposta:

- `200 OK` — `data`: objeto com os campos:

  - `event`: nome do evento (string).
  - `contest`: nome do contest (string); `""` é o contest padrão.
  - `problems`: lista de letras dos problemas (strings unicode).
  - `teams`: lista de times do contest, cada um com `login`, `escola` e `nome` (strings).
  - `time_seconds`: tempo decorrido, em segundos.
  - `score_freeze_time_seconds`: instante do congelamento do placar, em segundos.
  - `penalty_seconds`: penalidade por submissão incorreta, em segundos.

- `403 Forbidden` — `errors`: `[{ "code": "not_started", ... }]` (o evento ainda não começou).
- `404 Not Found` — `errors`: `[{ "code": "not_found", ... }]` (evento ou contest inexistente).

Exemplo:

```json
{
    "data": {
        "event": "regional-2026",
        "contest": "brasil",
        "problems": ["A", "B", "C", "D"],
        "teams": [
            { "login": "teambrmscg001", "escola": "FACOM - UFMS", "nome": "Time de Teste" }
        ],
        "time_seconds": 3218,
        "score_freeze_time_seconds": 2040,
        "penalty_seconds": 1200
    }
}
```

### Config público do contest

- `GET /api/events/{event-name}/contests/{contest-name}/config`
- Configuração pública do contest: campos do contest e seus sites, sem `salt` em nenhum nível. Usada pelo cliente para seleção de sede, estilos, medalhas e para as URLs de fotos e sons. Antes do início, o endpoint responde `403 not_started`.

Resposta:

- `200 OK` — `data`: objeto com os campos:

  - `name`: nome do contest (string).
  - `codes`: lista de expressões regulares que casam com o login dos times do contest.
  - `style`: nome do estilo visual do contest; opcional.
  - `ouro`, `prata`, `bronze`: posições de medalha (1-based); opcionais, padrões `1`, `2`, `3`.
  - `sites`: lista de sites do contest, cada um com `name` e `codes`; opcional.
  - `photo_url_format`: formato de URL das fotos do contest, com o placeholder `{team_login}`; opcional. Sem formato definido, vale o padrão relativo `photos/{team_login}.webp`, resolvido contra a mesma origem da API.
  - `sound_url_format`: formato de URL dos sons do contest, com o placeholder `{team_login}`; opcional. Sem formato definido, vale o padrão relativo `sounds/{team_login}.mp3`, resolvido contra a mesma origem da API.

- `403 Forbidden` — `errors`: `[{ "code": "not_started", ... }]` (o evento ainda não começou).
- `404 Not Found` — `errors`: `[{ "code": "not_found", ... }]`.

Exemplo:

```json
{
    "data": {
        "name": "brasil",
        "codes": ["teambr"],
        "style": "brasil",
        "ouro": 4,
        "prata": 8,
        "bronze": 12,
        "sites": [
            { "name": "fiemg", "codes": ["teammg"] }
        ],
        "photo_url_format": "https://static.example.com/photos/{team_login}.webp",
        "sound_url_format": "https://static.example.com/sounds/{team_login}.mp3"
    }
}
```

### Stream público de runs

- `WS /api/events/{event-name}/contests/{contest-name}/runs_ws`
- Stream ao vivo de runs. Ao conectar, o cliente recebe as runs do contest já recebidas desde a criação do evento e, em seguida, as novas conforme chegam. Somente runs de times cujo login casa com `codes` do contest são enviadas; **cabe ao cliente aplicar o congelamento** (`score_freeze_time_seconds`) na exibição.

Mensagens: objetos JSON, um por mensagem:

- `id`: identificador da submissão (inteiro); um `id` repetido corrige o resultado anterior — o último valor é o considerado.
- `team_login`: login do time (string).
- `prob`: letra do problema (string unicode).
- `time_seconds`: instante da submissão, em segundos.
- `answer`: resultado, um de `"Y"`, `"N"`, `"?"` ou `"X"`.

Handshake:

- `101 Switching Protocols` — conexão estabelecida.
- `403 Forbidden` — o evento ainda não começou (sem corpo).
- `404 Not Found` — evento ou contest inexistente (sem corpo).

Exemplo de mensagem:

```json
{ "id": 1, "team_login": "teambrmscg001", "prob": "A", "time_seconds": 56, "answer": "Y" }
```

### Runs secretas de um site

- `GET /api/events/{event-name}/contests/{contest-name}/runs_secret`
- Todas as runs do site (incluindo as congeladas), para a revelação. O site é identificado pela **chave**: o servidor compara a chave recebida com as chaves derivadas dos sites do contest (ver [event-api.md](event-api.md), seção Salts) e casa com o site correspondente.
- Cabeçalho obrigatório: `Authorization: Bearer <site-key>`. A chave não vai na URL (evita vazamento em logs).
- Site sem `salt` próprio não tem chave derivada e, portanto, nenhuma chave funciona.

Resposta:

- `200 OK` — `data`: objeto com o campo `runs`, lista de runs do site, no formato da run acima.
- `403 Forbidden` — `errors`: `[{ "code": "not_started", ... }]` (o evento ainda não começou; nenhuma chave funciona antes do início).
- `403 Forbidden` — `errors`: `[{ "code": "invalid_key", ... }]` (chave ausente ou não casa com nenhum site do contest).
- `404 Not Found` — `errors`: `[{ "code": "not_found", ... }]`.

Exemplo:

```json
{
    "data": {
        "runs": [
            { "id": 1, "team_login": "teambrmscg001", "prob": "A", "time_seconds": 56, "answer": "Y" },
            { "id": 2, "team_login": "teambrmscg001", "prob": "B", "time_seconds": 139, "answer": "N" }
        ]
    }
}
```

### Timer do evento

- `WS /api/events/{event-name}/timer`
- Stream do relógio do evento. Independe de contest. Disponível antes do início (alimenta o countdown do cliente).

Mensagens: objetos JSON, um por mensagem, com duplicatas consecutivas suprimidas:

- `current_time_seconds`: tempo corrente, em segundos; pode ser negativo (countdown anterior ao início).
- `score_freeze_time_seconds`: instante do congelamento do placar, em segundos.

Handshake:

- `101 Switching Protocols` — conexão estabelecida.
- `404 Not Found` — evento inexistente (sem corpo).

Exemplo de mensagem:

```json
{ "current_time_seconds": 3218, "score_freeze_time_seconds": 2040 }
```

### Controle remoto

- `WS /api/events/{event-name}/contests/{contest-name}/remote_control/{key}`
- Relay de mensagens de controle entre as abas/browsers que usam a mesma chave, isolado por contest. Cada mensagem recebida de um cliente é retransmitida a todos os outros clientes da mesma chave; o remetente não recebe a própria mensagem.

Mensagens: frames de texto com um dos objetos abaixo (JSON):

- `{ "WindowScroll": { "y": <posição> } }` — sincroniza a rolagem da janela.
- `{ "QueryString": { "query": <string> } }` — sincroniza a query string (ex.: troca de sede).
- `{ "PhotoState": "Hidden" }` ou `{ "PhotoState": { "Show": <team_login> } }` — sincroniza a foto exibida.

Handshake:

- `101 Switching Protocols` — conexão estabelecida.
- `404 Not Found` — evento ou contest inexistente (sem corpo).

### Métricas

- `GET /api/metrics`
- Métricas do processo no formato texto do Prometheus (autometrics). Global: não é por evento.

Resposta:

- `200 OK` — corpo em texto Prometheus, **sem envelope** (não é JSON).
- `500 Internal Server Error` — falha ao codificar as métricas.

## Resumo das regras

- Todos os endpoints públicos ficam sob `/api`, espelhando a hierarquia interna.
- Todos os tempos em segundos, com a unidade no nome (`*_seconds`); `time_seconds`/`current_time_seconds` podem ser negativos (countdown anterior ao início).
- Antes do início (`time_seconds < 0`), os endpoints do contest respondem `403 not_started` — o cliente mostra a tela de countdown usando o timer do evento. A lista de eventos, o timer e o controle remoto continuam disponíveis.
- Sem autenticação, exceto `runs_secret` (chave do site via `Authorization: Bearer`).
- Nada sensível no escopo público: sem `salt`, sem chaves derivadas.
- Fotos e sons vêm do config do contest (`GET .../config`), não do estado nem do `config.json`.
- Respostas JSON usam o envelope `{ data, errors, warnings }` (campos opcionais); WebSockets respondem só com o status do handshake e trocam payloads nus.
- O contest padrão (`""`) usa segmento vazio no caminho.
- A chave do site identifica o site; a troca do salt do site (via API interna) troca a chave imediatamente.
