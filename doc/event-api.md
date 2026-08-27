# API REST de eventos

Esta API substitui o antigo arquivo webcast. Todos os tempos são expressos em **segundos**, sem exceção.

## Escopo

Todos os endpoints desta API ficam sob o escopo `/internal`.

## Autenticação

Todos os endpoints são privados e exigem autenticação HTTP Basic com um token:

- Cabeçalho: `Authorization: Basic <base64(usuario:token)>`.
- Sem credenciais válidas: `401 Unauthorized`.

## Envelope de resposta

Toda resposta com corpo JSON é um objeto com os campos `data`, `errors` e `warnings`. Os três campos são **opcionais** e ausentes quando vazios:

- `data`: recurso ou resultado da operação; presente apenas em respostas de sucesso (2xx).
- `errors`: lista de objetos `{ "code": <string>, "message": <string> }`; presente apenas em respostas de erro (4xx/5xx).
- `warnings`: lista de objetos `{ "code": <string>, "message": <string> }`; problemas não fatais, presentes somente junto com `data`.

Respostas de sucesso nunca trazem `errors`; respostas de erro nunca trazem `data`. Os códigos HTTP continuam valendo — o envelope acrescenta detalhe, não os substitui. `204 No Content` não tem corpo (sem envelope). Corpos de **requisição** não usam o envelope.

Salvo indicação contrária, respostas de erro trazem `errors` com o código canônico da situação:

| code | status | situação |
| --- | --- | --- |
| `invalid_json` | 400 | JSON malformado |
| `missing_field` | 400 | campo obrigatório ausente |
| `invalid_regex` | 400 | regex inválida em `codes` |
| `invalid_value` | 400 | valor inválido (tempo negativo, `answer` desconhecido etc.) |
| `unauthorized` | 401 | credenciais ausentes ou inválidas |
| `not_found` | 404 | recurso inexistente |
| `conflict` | 409 | criação de recurso já existente |

### Exemplos

```json
{
    "data": { "added": 3, "updated": 1 }
}
```

```json
{
    "errors": [
        { "code": "not_found", "message": "evento não existe" }
    ]
}
```

## Recursos

A API organiza os recursos em hierarquia: **events** → **contests** → **sites**.

- Evento (`/internal/events/{event-name}`): o contest como um todo — problemas, times, tempo e runs.
- Contest (`/internal/contests/{event-name}/{contest-name}`): agrupamento de times do evento, identificado por nome.
- Site (`/internal/sites/{event-name}/{contest-name}/{site-name}`): agrupamento de times de um contest, com chave própria (ver seção Salts).

## Estado do evento

O estado do evento é um objeto JSON com os seguintes campos:

- `name`: nome do evento (string).
- `problems`: lista de letras dos problemas (strings unicode); a ordem da lista define a letra de cada problema.
- `teams`: lista de times, cada um com os campos `login`, `escola` e `nome` (strings).
- `score_freeze_time`: instante do congelamento do placar, em segundos.
- `penalty`: penalidade por submissão incorreta, em segundos.
- `time`: tempo decorrido, em segundos.
- `salt`: string usada para derivar as chaves dos sites (ver seção Salts); opcional.

Não há campo de duração, tempo corrente declarado ou contagem de times: a contagem é derivada da lista de times.

### Exemplo

```json
{
    "name": "ENSAIO - Maratona 2026",
    "problems": ["A", "B", "C", "D"],
    "teams": [
        { "login": "teambrmscg001", "escola": "FACOM - UFMS", "nome": "Time de Teste" }
    ],
    "score_freeze_time": 2040,
    "penalty": 1200,
    "time": 3218,
    "salt": "s3gredo-do-evento"
}
```

## Endpoints do evento

### Criar o evento

- `POST /internal/events/{event-name}`
- Corpo: estado do evento; `time` é opcional e assume `0`.

Respostas:

- `201 Created` — `data`: estado do evento como armazenado.
- `400 Bad Request` — corpo inválido (JSON malformado ou campo obrigatório ausente).
- `401 Unauthorized`.
- `409 Conflict` — o evento já existe.

### Ler o evento

- `GET /internal/events/{event-name}`

Respostas:

- `200 OK` — `data`: estado atual do evento.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Atualizar todos os valores do evento

- `PUT /internal/events/{event-name}`
- Corpo: estado completo do evento.

Respostas:

- `200 OK` — `data`: estado atualizado.
- `400 Bad Request` — corpo inválido.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Atualizar somente o tempo

- `PATCH /internal/events/{event-name}/time`
- Corpo: `{ "time": <segundos> }`.

Respostas:

- `200 OK` — `data`: `{ "time": <novo valor> }`.
- `400 Bad Request` — corpo inválido ou tempo negativo.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Remover o evento

- `DELETE /internal/events/{event-name}`
- Remove o evento, seus contests, sites e todas as runs.

Respostas:

- `204 No Content` — sem corpo.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

## Contests

Um contest é um agrupamento de times do evento, identificado por um nome. Múltiplos contests podem existir no mesmo evento. O contest de nome `""` é o contest padrão — aquele mapeado para a consulta vazia (`?contest=`).

### Formato de um contest

- `name`: nome do contest (string); obrigatório. `""` representa o contest padrão.
- `codes`: lista de expressões regulares que casam com o login dos times pertencentes ao contest; obrigatório.
- `salt`: string usada para derivar as chaves dos sites deste contest (ver seção Salts); opcional.
- `style`: nome do estilo visual do contest; opcional.
- `ouro`: posição até a qual vale medalha de ouro (1-based); opcional, padrão `1`.
- `prata`: idem para prata; opcional, padrão `2`.
- `bronze`: idem para bronze; opcional, padrão `3`.
- `photo_url_format`: formato de URL das fotos do contest (ver seção Mídia); opcional.
- `sound_url_format`: formato de URL dos sons do contest (ver seção Mídia); opcional.

Chaves não listadas aqui são ignoradas.

### Exemplo

```json
{
    "name": "brasil",
    "codes": ["teambr"],
    "salt": "s3gredo-do-contest",
    "style": "brasil",
    "ouro": 4,
    "prata": 8,
    "bronze": 12,
    "photo_url_format": "https://static.example.com/photos/{team_login}.webp",
    "sound_url_format": "https://static.example.com/sounds/{team_login}.mp3"
}
```

### Criar um contest

- `POST /internal/contests/{event-name}/{contest-name}`
- Corpo: contest (formato acima).

Respostas:

- `201 Created` — `data`: contest como armazenado.
- `400 Bad Request` — corpo inválido, `codes` ausente ou regex inválida.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.
- `409 Conflict` — já existe um contest com esse nome.

### Substituir um contest

- `PUT /internal/contests/{event-name}/{contest-name}`
- Corpo: contest completo (substitui todos os valores).

Respostas:

- `200 OK` — `data`: contest atualizado.
- `400 Bad Request` — corpo inválido.
- `401 Unauthorized`.
- `404 Not Found` — o evento ou o contest não existe.

### Remover um contest

- `DELETE /internal/contests/{event-name}/{contest-name}`
- Remove também os sites do contest.
- Para o contest padrão, o segmento `{contest-name}` é vazio: `DELETE /internal/contests/{event-name}/`.

Respostas:

- `204 No Content` — sem corpo.
- `401 Unauthorized`.
- `404 Not Found` — o evento ou o contest não existe.

## Sites

Um site é um agrupamento de times de um contest, identificado por um nome — tipicamente a sede física que exibe o placar. Cada site tem sua própria chave para as runs secretas (ver seção Salts). Para o contest padrão (`name = ""`), o segmento `{contest-name}` é vazio: `/internal/sites/{event-name}//{site-name}`.

### Formato de um site

- `name`: nome do site (string); obrigatório.
- `codes`: lista de expressões regulares que casam com o login dos times do site; obrigatório.
- `salt`: string usada para derivar a chave do site (ver seção Salts); opcional.

Chaves não listadas aqui são ignoradas.

### Exemplo

```json
{
    "name": "fiemg",
    "codes": ["teammg"],
    "salt": "s3gredo-do-site"
}
```

### Criar um site

- `POST /internal/sites/{event-name}/{contest-name}/{site-name}`
- Corpo: site (formato acima).

Respostas:

- `201 Created` — `data`: site como armazenado.
- `400 Bad Request` — corpo inválido, `codes` ausente ou regex inválida.
- `401 Unauthorized`.
- `404 Not Found` — o evento ou o contest não existe.
- `409 Conflict` — já existe um site com esse nome.

### Substituir um site

- `PUT /internal/sites/{event-name}/{contest-name}/{site-name}`
- Corpo: site completo (substitui todos os valores).

Respostas:

- `200 OK` — `data`: site atualizado.
- `400 Bad Request` — corpo inválido.
- `401 Unauthorized`.
- `404 Not Found` — o evento, o contest ou o site não existe.

### Remover um site

- `DELETE /internal/sites/{event-name}/{contest-name}/{site-name}`

Respostas:

- `204 No Content` — sem corpo.
- `401 Unauthorized`.
- `404 Not Found` — o evento, o contest ou o site não existe.

## Salts

Cada nível da hierarquia tem um salt opcional: o evento, cada contest e cada site. As chaves dos sites são **derivadas** desses salts; não há envio de chaves.

- Chave de um site: `key(site) = HMAC-SHA256(salt_evento : salt_contest : salt_site, contest_name : site_name)`, codificada em base62 e truncada em 12 caracteres. O `:` é um separador literal; salt ausente contribui com string vazia na sua posição.
- Site sem `salt` próprio não tem chave (revelação desabilitada para aquele site).
- Dois sites exibindo o mesmo contest têm chaves distintas, pois o salt do site entra na derivação.
- Alcance da troca de salt: trocar o salt de um site muda somente a chave daquele site; trocar o salt de um contest muda as chaves de todos os seus sites; trocar o salt do evento muda todas as chaves do evento.
- Para remover um salt, atualize o recurso inteiro (`PUT`) sem o campo `salt`.

### Trocar o salt

- `POST /internal/events/{event-name}/salt`
- `POST /internal/contests/{event-name}/{contest-name}/salt`
- `POST /internal/sites/{event-name}/{contest-name}/{site-name}/salt`

Corpo opcional: `{ "salt": "<novo valor>" }`. Se o corpo ou o campo `salt` estiver ausente ou vazio, o servidor gera um salt aleatório. O restante do recurso não é alterado.

Respostas:

- `200 OK` — `data`: `{ "salt": "<valor efetivo>" }`.
- `400 Bad Request` — corpo inválido.
- `401 Unauthorized`.
- `404 Not Found` — o evento, o contest ou o site não existe.

## Mídia

Fotos e sons não são montados como volumes; cada contest aceita formatos de URL:

- `photo_url_format`: string com o placeholder `{team_login}`; opcional.
- `sound_url_format`: string com o placeholder `{team_login}`; opcional.
- Sem formato definido, valem os padrões relativos `photos/{team_login}.webp` e `sounds/{team_login}.mp3`, resolvidos contra a mesma origem da API.

## Runs

Runs são enviadas separadamente, depois da criação do evento, e adicionadas às runs existentes.

### Formato de uma run

- `id`: identificador da submissão (inteiro).
- `team_login`: login do time (string).
- `prob`: letra do problema (string unicode, conforme a lista de problemas do evento).
- `time`: instante da submissão, em segundos.
- `answer`: resultado, um de `"Y"`, `"N"`, `"?"` ou `"X"`.

### Exemplo

```json
{
    "runs": [
        { "id": 1, "team_login": "teambrmscg001", "prob": "A", "time": 56, "answer": "Y" },
        { "id": 2, "team_login": "teambrmscg001", "prob": "B", "time": 139, "answer": "N" }
    ]
}
```

### Adicionar runs

- `POST /internal/events/{event-name}/runs`
- Corpo: `{ "runs": [ ... ] }`.
- Aplica as runs às existentes, na ordem em que aparecem no corpo: um `id` novo adiciona a submissão; um `id` já existente substitui o resultado anterior — o último valor é o considerado (correção do juiz).

Respostas:

- `200 OK` — `data`: `{ "added": <quantidade>, "updated": <quantidade> }`, com a quantidade de submissões novas e de resultados corrigidos, respectivamente.
- `400 Bad Request` — corpo inválido, `answer` fora de `"Y" | "N" | "?" | "X"`, ou `team_login`/`prob` desconhecidos.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Remover todas as runs

- `DELETE /internal/events/{event-name}/runs`

Respostas:

- `204 No Content` — sem corpo.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

## Códigos de resposta comuns

- `200 OK` — operação concluída; `data` com o recurso ou resultado.
- `201 Created` — recurso criado; `data` com o recurso criado.
- `204 No Content` — remoção concluída; sem corpo.
- `400 Bad Request` — corpo inválido (JSON malformado, campos ausentes ou com valores inválidos); `errors`.
- `401 Unauthorized` — credenciais ausentes ou inválidas; `errors`.
- `404 Not Found` — evento, contest, site ou runs inexistentes; `errors`.
- `409 Conflict` — criação de recurso já existente; `errors`.

## Resumo das regras

- Todos os endpoints ficam sob `/internal`.
- Todos os tempos em segundos.
- Todos os endpoints exigem autenticação HTTP Basic com token.
- Hierarquia de recursos: events → contests → sites.
- Toda resposta com corpo JSON usa o envelope `{ data, errors, warnings }` (campos opcionais); `204` não tem corpo.
- Runs são enviadas somente após a criação do evento.
- Envios de runs são incrementais; um `id` repetido corrige o resultado da submissão (o último valor é o considerado).
- Atualizações completas via `PUT`; atualização de tempo via `PATCH /internal/events/{event-name}/time`.
- Salts opcionais nos três níveis (evento, contest, site); as chaves dos sites são derivadas dos três salts (HMAC-SHA256, base62, 12 caracteres) e trocadas via `POST .../salt`.
- Mídia é configurada por formatos de URL, não por volumes.
