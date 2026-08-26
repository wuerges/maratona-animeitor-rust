# API REST de eventos

Esta API substitui o antigo arquivo webcast. Todos os tempos são expressos em **segundos**, sem exceção.

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
    "time": 3218
}
```

## Endpoints

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
- Remove o evento e todas as suas runs.
- Resposta: `204 No Content`.

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
- Runs são enviadas somente após a criação do evento.
- Envios de runs são incrementais (append) e idempotentes por `id`.
- Atualizações completas via `PUT /event`; atualização de tempo via `PATCH /event/time`.
