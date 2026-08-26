# API REST de eventos

Esta API substitui o antigo arquivo webcast. Todos os tempos são expressos em **segundos**, sem exceção.

## Autenticação

Todos os endpoints desta API são privados e exigem autenticação HTTP Basic com um token:

- Cabeçalho: `Authorization: Basic <base64(usuario:token)>`.
- Sem credenciais válidas: `401 Unauthorized`.

## Recurso raiz

- `/event`

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

- `POST /event`
- Corpo: estado do evento; `time` é opcional e assume `0`.
- Resposta: `201 Created`. Se o evento já existe: `409 Conflict`.

### Ler o evento

- `GET /event`
- Resposta: estado atual do evento.

### Atualizar todos os valores do evento

- `PUT /event`
- Corpo: estado completo do evento.
- Resposta: `200 OK`. Se o evento não existe: `404 Not Found`.

### Atualizar somente o tempo

- `PATCH /event/time`
- Corpo: `{ "time": <segundos> }`.
- Resposta: `200 OK`. Se o evento não existe: `404 Not Found`.

### Remover o evento

- `DELETE /event`
- Remove o evento, seus contests e todas as runs.
- Resposta: `204 No Content`.

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

- `POST /event/contest`
- Corpo: contest (formato acima).
- Resposta: `201 Created`. Se o evento não existe: `404 Not Found`. Se o contest já existe: `409 Conflict`.

### Substituir um contest

- `PUT /event/contest`
- Corpo: contest completo (substitui todos os valores).
- Resposta: `200 OK`. Se o evento não existe: `404 Not Found`.

### Remover um contest

- `DELETE /event/contest/{name}`
- Para o contest padrão, o segmento `{name}` é vazio: `DELETE /event/contest/`.
- Resposta: `204 No Content`.

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

- `POST /event/runs`
- Corpo: `{ "runs": [ ... ] }`.
- Adiciona as runs às existentes; submissões com o mesmo `id` são ignoradas (idempotente).
- Resposta: `200 OK`. Se o evento não existe: `404 Not Found`.

### Remover todas as runs

- `DELETE /event/runs`
- Resposta: `204 No Content`.

## Resumo das regras

- Todos os tempos em segundos.
- Todos os endpoints exigem autenticação HTTP Basic com token.
- Runs são enviadas somente após a criação do evento.
- Envios de runs são incrementais (append) e idempotentes por `id`.
- Atualizações completas via `PUT`; atualização de tempo via `PATCH /event/time`.
- Chaves dos contests são derivadas do `salt` do evento (HMAC-SHA256 truncado em 12 caracteres base62).
- Mídia é configurada por formatos de URL, não por volumes.
