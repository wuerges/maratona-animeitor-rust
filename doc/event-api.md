# API REST de eventos

Esta API substitui o antigo arquivo webcast. Todos os tempos são expressos em **segundos**, sem exceção, e a unidade faz parte do nome do campo (ex.: `score_freeze_time_seconds`).

A especificação executável e o roteiro completo em inglês estão em `/internal/openapi.json` e `/internal/docs` (autenticados). O roteiro também está em [internal-api-setup.md](internal-api-setup.md).

## Escopo

Todos os endpoints desta API ficam sob o escopo `/internal`.

## Autenticação

Todos os endpoints são privados e exigem autenticação HTTP Basic com um token:

- Cabeçalho: `Authorization: Basic <base64(usuario:token)>`. Tanto o usuário quanto seu token devem corresponder a uma credencial habilitada no servidor.
- Use HTTPS; o listener HTTP responde `426` em texto, sem envelope.
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
| `invalid_value` | 400 | valor inválido (ex.: `answer` desconhecido) |
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

- `name`: identificador do evento (string); deve ser igual ao nome no caminho da requisição.
- `problems`: lista de letras dos problemas (strings unicode); a ordem da lista define a letra de cada problema.
- `teams`: lista de times, cada um com os campos `login`, `escola` e `nome` (strings).
- `score_freeze_time_seconds`: instante do congelamento do placar, em segundos.
- `penalty_seconds`: penalidade por submissão incorreta, em segundos.
- `time_seconds`: tempo decorrido, em segundos; pode ser negativo (countdown anterior ao início).
- `salt`: string usada para derivar as chaves dos sites (ver seção Salts); opcional.

Não há campo de duração, tempo corrente declarado ou contagem de times: a contagem é derivada da lista de times. Antes do início (`time_seconds < 0`), os endpoints públicos do contest respondem `403 not_started`; a API interna devolve sempre o estado completo. O servidor não avança o relógio automaticamente: o controlador/feeder deve atualizar `time_seconds`. O estado fica em memória e é perdido no reinício.

### Exemplo

```json
{
    "name": "ensaio-2026",
    "problems": ["A", "B", "C", "D"],
    "teams": [
        { "login": "teambrmscg001", "escola": "FACOM - UFMS", "nome": "Time de Teste" }
    ],
    "score_freeze_time_seconds": 2040,
    "penalty_seconds": 1200,
    "time_seconds": 3218,
    "salt": "s3gredo-do-evento"
}
```

## Endpoints do evento

### Criar o evento

- `POST /internal/events/{event-name}`
- Corpo: estado do evento; `time_seconds` é opcional e assume `0`.

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
- Corpo: estado completo do evento. Preserva contests, sites e runs; campos opcionais omitidos voltam ao padrão (`time_seconds: 0`, `salt: null`).

Respostas:

- `200 OK` — `data`: estado atualizado.
- `400 Bad Request` — corpo inválido.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Atualizar somente o tempo

- `PATCH /internal/events/{event-name}/time`
- Corpo: `{ "time_seconds": <segundos> }`. Valores negativos são permitidos (countdown anterior ao início).

Respostas:

- `200 OK` — `data`: `{ "time_seconds": <novo valor> }`.
- `400 Bad Request` — corpo inválido.
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

Um contest é um agrupamento de times do evento, identificado por um nome não-vazio. Múltiplos contests podem existir no mesmo evento.

### Formato de um contest

- `name`: nome do contest (string); obrigatório, não-vazio e igual ao nome no caminho.
- `codes`: lista de expressões regulares que casam com o login dos times pertencentes ao contest; obrigatório.
- `salt`: string usada para derivar as chaves dos sites deste contest (ver seção Salts); opcional.
- `style`: nome do estilo visual do contest; opcional.
- `ouro`: posição até a qual vale medalha de ouro (1-based); opcional, padrão `1`.
- `prata`: idem para prata; opcional, padrão `2`.
- `bronze`: idem para bronze; opcional, padrão `3`.
- `photo_url_format`: formato de URL das fotos do contest (ver seção Mídia); opcional.
- `sound_url_format`: formato de URL dos sons do contest (ver seção Mídia); opcional.

`codes` combina regexes Rust por OR, sem ancoragem automática; `[]` não seleciona times e `[".*"]` seleciona todos.

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
- `400 Bad Request` — corpo inválido, nome vazio, `codes` ausente ou regex inválida.
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

Respostas:

- `204 No Content` — sem corpo.
- `401 Unauthorized`.
- `404 Not Found` — o evento ou o contest não existe.

## Sites

Um site é um agrupamento de times de um contest, identificado por um nome — tipicamente a sede física que exibe o placar. Cada site tem sua própria chave para as runs secretas (ver seção Salts). Configure os filtros como subconjunto dos times do contest: as runs secretas usam os filtros do site sobre as runs do evento, sem impor essa interseção.

### Formato de um site

- `name`: nome do site (string); obrigatório.
- `codes`: lista de expressões regulares que casam com o login dos times do site; obrigatório.
- `salt`: string usada para derivar a chave do site (ver seção Salts); opcional.

`codes` combina regexes Rust por OR, sem ancoragem automática; `[]` não seleciona times e `[".*"]` seleciona todos.

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

### Métricas

- `GET /internal/metrics` (autenticado, como todo o escopo interno)
- Métricas do processo no formato texto do Prometheus (autometrics). Global: não é por evento.

Resposta:

- `200 OK` — corpo em texto Prometheus, **sem envelope** (não é JSON).
- `500 Internal Server Error` — falha ao codificar as métricas.

### Remover um site

- `DELETE /internal/sites/{event-name}/{contest-name}/{site-name}`

Respostas:

- `204 No Content` — sem corpo.
- `401 Unauthorized`.
- `404 Not Found` — o evento, o contest ou o site não existe.

## Consultar configurações e URLs de revelação

- `GET /internal/events`: nomes dos eventos em ordem de criação.
- `GET /internal/events/{event-name}/contests`: configurações completas dos contests, com salts, em ordem não especificada.
- `GET /internal/events/{event-name}/contests/{contest-name}/sites`: configurações completas dos sites, com salts, em ordem não especificada.
- `GET /internal/contests/{event-name}/{contest-name}` e `GET /internal/sites/{event-name}/{contest-name}/{site-name}` retornam configurações individuais.
- `GET /internal/events/{event-name}/revelation_urls`: URLs completas de todos os sites do evento, ordenadas por contest e site. Funciona antes do início e exige a mesma autenticação interna.

Exemplo de resposta `200`, com `Cache-Control: no-store`:

```json
{"data":[{"contest":"brasil","site":"fiemg","url":"https://example.com/animeitor/regional-2026/brasil/?secret=EXAMPLE_KEY&sede=fiemg"}]}
```

Evento sem sites retorna `{"data":[]}`; evento inexistente retorna `404 not_found`; credenciais inválidas retornam `401 unauthorized`. A origem vem de `public_url`; o caminho configurado é substituído por `/animeitor/{evento}/{contest}/`, como no `printurls`. O frontend deve estar publicado nessa origem.

O parâmetro `secret` contém a chave usada como Bearer em `runs_secret`; `sede` seleciona o site. Não há endpoint separado para consultar chaves. As URLs são credenciais privadas: não publique junto dos links públicos. Após trocar salts, consulte novamente as URLs afetadas.

## Salts

Cada nível da hierarquia tem um salt opcional: o evento, cada contest e cada site. As chaves dos sites são **derivadas** desses salts; não há envio de chaves.

- Chave de um site: HMAC-SHA256 usando `revelation_salt` privado do servidor como chave. A mensagem é o array JSON compacto `["animeitor-site-key-v1", event_name, contest_name, site_name, salt_evento, salt_contest, salt_site]`. O resultado é codificado em base62 e truncado em 12 caracteres; valores ausentes contribuem strings vazias. Chaves legadas sem o segredo do servidor são rejeitadas.
- Os salts da hierarquia podem ser públicos; a derivação sempre exige o `revelation_salt` privado, que nunca é exposto pela API. Alterá-lo troca todas as chaves do servidor.
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
- `time_seconds`: instante da submissão, em segundos.
- `answer`: resultado, um de `"Y"`, `"N"`, `"?"` ou `"X"`.

### Exemplo

```json
{
    "runs": [
        { "id": 1, "team_login": "teambrmscg001", "prob": "A", "time_seconds": 56, "answer": "Y" },
        { "id": 2, "team_login": "teambrmscg001", "prob": "B", "time_seconds": 139, "answer": "N" }
    ]
}
```

### Adicionar runs

- `POST /internal/events/{event-name}/runs`
- Corpo: `{ "runs": [ ... ] }`.
- Ignora times desconhecidos com warnings e ordena as demais runs por `(time_seconds, id)` antes de validar os problemas e aplicar. Um `id` novo adiciona a submissão; um `id` existente com campos alterados corrige a submissão; reenvios idênticos não alteram nem incrementam `updated`. Entradas com a mesma chave de ordenação preservam a ordem do corpo.

Respostas:

- `200 OK` — `data`: `{ "added": <quantidade>, "updated": <quantidade> }`, com a quantidade de submissões novas e de resultados corrigidos, respectivamente.
- `400 Bad Request` — corpo inválido, `answer` fora de `"Y" | "N" | "?" | "X"`, ou `prob` desconhecido. Runs de `team_login` que não está no evento (ex.: usuários juízes do feed do MOJ) são ignoradas e reportadas em `warnings` (`code: "unknown_team"`), sem rejeitar o lote.
- `401 Unauthorized`.
- `404 Not Found` — o evento não existe.

### Remover todas as runs

- `DELETE /internal/events/{event-name}/runs`
- Limpa as runs armazenadas, mas mantém o histórico de replay do WebSocket e não envia mensagem de reset. Reconexões podem receber runs antigas; para limpar também esse histórico, recrie o evento e sua configuração.

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
- Todos os tempos em segundos, com a unidade no nome (`*_seconds`); `time_seconds` pode ser negativo (countdown).
- Todos os endpoints exigem autenticação HTTP Basic com token.
- Hierarquia de recursos: events → contests → sites.
- Toda resposta com corpo JSON usa o envelope `{ data, errors, warnings }` (campos opcionais); `204` não tem corpo.
- Runs são enviadas somente após a criação do evento.
- Envios de runs são incrementais; um `id` repetido corrige o resultado da submissão (o último valor é o considerado).
- Atualizações completas via `PUT`; atualização de tempo via `PATCH /internal/events/{event-name}/time`.
- Salts opcionais nos três níveis (evento, contest, site); as chaves dos sites são derivadas do segredo privado do servidor e dos três salts públicos (HMAC-SHA256, base62, 12 caracteres) e trocadas via `POST .../salt`.
- Mídia é configurada por formatos de URL, não por volumes.

## Operações incrementais

PATCH nos caminhos de evento, contest e site altera apenas os campos enviados, atomicamente; arrays substituem a lista inteira. `null` limpa campos opcionais, mas é inválido para campos obrigatórios. Nomes não podem mudar. Campos desconhecidos e patches vazios são rejeitados.

Times: `POST .../events/{evento}/teams` adiciona `{login,escola,nome}`; GET, PATCH e DELETE em `.../teams/{login}` consultam, editam nome/escola e removem. Problemas: `POST .../events/{evento}/problems` adiciona `{"problem":"C"}`; DELETE em `.../problems/{problema}` remove.

Remoção de item com runs retorna `409 conflict`. Para times, `?keep_runs=true` permite remover preservando as runs (também no PATCH de evento substituindo `teams`). Runs preservadas continuam no armazenamento e no replay; podem aparecer em streams filtrados por regex. Não há essa opção para problemas. PUT mantém o comportamento anterior, sem essas verificações novas.

PATCH em `.../contests/{evento}/{contest}/codes` ou `.../sites/{evento}/{contest}/{site}/codes` recebe `{"add":["regex"],"remove":["regex-antiga"]}`. As strings são comparadas exatamente; adições existentes e remoções ausentes não alteram nada. Toda validação ocorre antes da alteração.

Veja [o contrato completo e exemplos](internal-api-setup.md#atomic-incremental-management), também incluídos na especificação OpenAPI interna. O feeder mantém seu comportamento e pode sobrescrever mudanças manuais a partir da fonte.
