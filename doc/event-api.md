# API REST de eventos

Esta API substitui o antigo arquivo webcast. Todos os tempos são expressos em **segundos**, sem exceção.

## Escopo

Todos os endpoints desta API ficam sob o escopo `/internal`.

## Autenticação

Todos os endpoints são privados e exigem autenticação HTTP Basic com um token:

- Cabeçalho: `Authorization: Basic <base64(usuario:token)>`.
- Sem credenciais válidas: `401 Unauthorized`.

## Recurso raiz

- `/internal/event`

## Estado do evento

O estado do evento é um objeto JSON com os seguintes campos:

- `name`: nome do evento (string).
- `problems`: lista de letras dos problemas (strings unicode); a ordem da lista define a letra de cada problema.
- `teams`: lista de times, cada um com os campos `login`, `escola` e `nome` (strings).
- `score_freeze_time`: instante do congelamento do placar, em segundos.
- `penalty`: penalidade por submissão incorreta, em segundos.
- `time`: tempo decorrido, em segundos.
- `salt`: string usada para derivar as chaves dos contests (ver seção Secrets); opcional.
- `photo_url_format`: formato de URL das fotos (ver seção Mídia); opcional.
- `sound_url_format`: formato de URL dos sons (ver seção Mídia); opcional.

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
    "salt": "s3gredo-do-evento",
    "photo_url_format": "https://static.example.com/photos/{team_login}.webp",
    "sound_url_format": "https://static.example.com/sounds/{team_login}.mp3"
}
```

## Endpoints do evento

### Criar o evento

- `POST /internal/event`
- Corpo: estado do evento; `time` é opcional e assume `0`.

Respostas:

- `201 Created` — evento criado; corpo: estado do evento como armazenado.
- `400 Bad Request` — corpo inválido (JSON malformado, campo obrigatório ausente ou regex inválida em `codes`).
- `401 Unauthorized` — credenciais ausentes ou inválidas.
- `409 Conflict` — o evento já existe.

### Ler o evento

- `GET /internal/event`

Respostas:

- `200 OK` — corpo: estado atual do evento.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Atualizar todos os valores do evento

- `PUT /internal/event`
- Corpo: estado completo do evento.

Respostas:

- `200 OK` — corpo: estado atualizado.
- `400 Bad Request` — corpo inválido.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Atualizar somente o tempo

- `PATCH /internal/event/time`
- Corpo: `{ "time": <segundos> }`.

Respostas:

- `200 OK` — corpo: `{ "time": <novo valor> }`.
- `400 Bad Request` — corpo inválido ou tempo negativo.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Remover o evento

- `DELETE /internal/event`
- Remove o evento, seus contests e todas as runs.

Respostas:

- `204 No Content` — removido.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

## Contests

Um contest é um agrupamento de times do evento, identificado por um nome. Múltiplos contests podem existir no mesmo evento. O contest de nome `""` é o contest padrão — aquele mapeado para a consulta vazia (`?contest=`).

### Formato de um contest

- `name`: nome do contest (string); obrigatório. `""` representa o contest padrão.
- `codes`: lista de expressões regulares que casam com o login dos times pertencentes ao contest; obrigatório.
- `style`: nome do estilo visual do contest; opcional.
- `ouro`: posição até a qual vale medalha de ouro (1-based); opcional, padrão `1`.
- `prata`: idem para prata; opcional, padrão `2`.
- `bronze`: idem para bronze; opcional, padrão `3`.

Chaves não listadas aqui são ignoradas.

### Exemplo

```json
{
    "name": "brasil",
    "codes": ["teambr"],
    "style": "brasil",
    "ouro": 4,
    "prata": 8,
    "bronze": 12
}
```

### Criar um contest

- `POST /internal/event/contest`
- Corpo: contest (formato acima).

Respostas:

- `201 Created` — corpo: contest como armazenado.
- `400 Bad Request` — corpo inválido, `name` ou `codes` ausentes, ou regex inválida.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.
- `409 Conflict` — já existe um contest com esse `name`.

### Substituir um contest

- `PUT /internal/event/contest`
- Corpo: contest completo (substitui todos os valores).

Respostas:

- `200 OK` — corpo: contest atualizado.
- `400 Bad Request` — corpo inválido.
- `401 Unauthorized`.
- `404 Not Found` — o evento ou o contest não existe.

### Remover um contest

- `DELETE /internal/event/contest/{name}`
- Para o contest padrão, o segmento `{name}` é vazio: `DELETE /internal/event/contest/`.

Respostas:

- `204 No Content` — removido.
- `401 Unauthorized`.
- `404 Not Found` — o evento ou o contest não existe.

## Secrets

As chaves dos contests são **derivadas** do `salt` do evento; não há envio de chaves por contest.

- Chave de um contest: `key(name) = HMAC-SHA256(salt, name)`, codificada em base62 e truncada em 12 caracteres.
- Um único `salt` gera chaves distintas para todos os contests do evento, incluindo o padrão (`name = ""`).
- Sem `salt` no evento, nenhuma chave é gerada (revelação desabilitada).

## Mídia

Fotos e sons não são montados como volumes; o evento aceita formatos de URL:

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

- `POST /internal/event/runs`
- Corpo: `{ "runs": [ ... ] }`.
- Adiciona as runs às existentes; submissões com o mesmo `id` são ignoradas (idempotente).

Respostas:

- `200 OK` — corpo: `{ "added": <quantidade> }`, com a quantidade de runs efetivamente adicionadas (duplicadas por `id` não contam).
- `400 Bad Request` — corpo inválido, `answer` fora de `"Y" | "N" | "?" | "X"`, ou `team_login`/`prob` desconhecidos.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Remover todas as runs

- `DELETE /internal/event/runs`

Respostas:

- `204 No Content` — removidas.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

## Códigos de resposta comuns

- `200 OK` — operação concluída; corpo com o recurso ou resultado.
- `201 Created` — recurso criado; corpo com o recurso criado.
- `204 No Content` — remoção concluída; sem corpo.
- `400 Bad Request` — corpo inválido (JSON malformado, campos ausentes ou com valores inválidos).
- `401 Unauthorized` — credenciais ausentes ou inválidas.
- `404 Not Found` — evento, contest ou runs inexistentes.
- `409 Conflict` — criação de recurso já existente.

## Resumo das regras

- Todos os endpoints ficam sob `/internal`.
- Todos os tempos em segundos.
- Todos os endpoints exigem autenticação HTTP Basic com token.
- Runs são enviadas somente após a criação do evento.
- Envios de runs são incrementais (append) e idempotentes por `id`.
- Atualizações completas via `PUT`; atualização de tempo via `PATCH /internal/event/time`.
- Chaves dos contests são derivadas do `salt` do evento (HMAC-SHA256 truncado em 12 caracteres base62).
- Mídia é configurada por formatos de URL, não por volumes.
