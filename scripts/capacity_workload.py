#!/usr/bin/env python3
"""Seed/update a dedicated capacity-test event over verified internal HTTPS.

Credentials are read from a server TOML file, never from command-line arguments.
The event must not exist for seed; this script never deletes existing events.
"""
import argparse
import base64
import json
import pathlib
import ssl
import time
import tomllib
import urllib.parse
import urllib.request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['seed', 'live', 'correct', 'freeze', 'snapshot'])
    parser.add_argument('--server-config', required=True, type=pathlib.Path)
    parser.add_argument('--event', default='capacity-test')
    parser.add_argument('--contest', default='all')
    parser.add_argument('--teams', type=int, default=2000)
    parser.add_argument('--runs', type=int, default=10000)
    parser.add_argument('--seconds', type=int, default=3600)
    parser.add_argument('--freeze-seconds', type=int, default=0)
    parser.add_argument('--freeze-at', type=int)
    parser.add_argument('--correct-at', type=int)
    parser.add_argument('--expected', required=True, type=pathlib.Path)
    parser.add_argument('--ledger', type=pathlib.Path, default=pathlib.Path('publish-ledger.jsonl'))
    args = parser.parse_args()
    config = tomllib.loads(args.server_config.read_text())
    token = next(t for t in config['tokens'] if t['name'] == config['client_token'] and t.get('enabled', True))
    base = config['server_url'].rstrip('/')
    url = urllib.parse.urlsplit(base)
    if url.scheme != 'https' or url.username or url.query or url.fragment:
        raise ValueError('server_url must be verified HTTPS without credentials/query/fragment')
    ca = args.server_config.parent / config['tls_ca_cert']
    context = ssl.create_default_context(cafile=str(ca))
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPSHandler(context=context))
    auth = 'Basic ' + base64.b64encode(f"{token['name']}:{token['token']}".encode()).decode()
    event = '/internal/events/' + urllib.parse.quote(args.event, safe='')
    state_path = args.expected.with_suffix(args.expected.suffix + '.publisher-state.json')
    known_runs = {} if args.mode == 'seed' else {r['id']: r for r in json.loads(state_path.read_text())}

    def save_runs(batch):
        known_runs.update((r['id'], r) for r in batch)
        temporary = state_path.with_suffix('.tmp')
        temporary.write_text(json.dumps(list(known_runs.values())))
        temporary.replace(state_path)

    def request(method, path, body=None):
        raw = None if body is None else json.dumps(body).encode()
        req = urllib.request.Request(base + path, raw, {'Authorization': auth, 'Content-Type': 'application/json'}, method=method)
        # Never retry mutations automatically: a timeout can be an unknown outcome.
        started = time.monotonic()
        with opener.open(req, timeout=30) as response:
            value = response.read()
            with args.ledger.with_suffix('.requests.jsonl').open('a') as log:
                log.write(json.dumps({'method': method, 'status': response.status, 'duration_ms': (time.monotonic() - started) * 1000}) + '\n')
            return json.loads(value).get('data') if value else None

    def snapshot():
        state = request('GET', event)
        runs = list(known_runs.values())
        public = []
        for run in runs:
            run = dict(run)
            if run['time_seconds'] >= state['score_freeze_time_seconds']:
                run['answer'] = '?'
            public.append(run)
        timer = {k: state[k] for k in ['score_freeze_time_seconds']}
        timer['current_time_seconds'] = state['time_seconds']
        # Obtain exact timer fields from state model: public timer has only these two.
        payload = {'runs': public, 'timer': timer}
        temp = args.expected.with_suffix(args.expected.suffix + '.tmp')
        temp.write_text(json.dumps(payload))
        temp.replace(args.expected)
        return state, runs

    def record_freeze(sent, state):
        with args.ledger.open('a') as ledger:
            for run in known_runs.values():
                public = dict(run)
                if public['time_seconds'] >= state['score_freeze_time_seconds']:
                    public['answer'] = '?'
                ledger.write(json.dumps({'kind': 'run', 'sent_unix_ms': sent, 'message': public}) + '\n')
            ledger.write(json.dumps({'kind': 'timer', 'sent_unix_ms': sent, 'message': {'current_time_seconds': state['time_seconds'], 'score_freeze_time_seconds': state['score_freeze_time_seconds']}}) + '\n')

    if args.teams <= 0 or args.runs <= 0 or not 0 < args.seconds <= 86400:
        raise ValueError('positive dataset sizes and a duration from 1 through 86400 seconds are required')
    if args.mode == 'seed':
        state = {'name': args.event, 'problems': list('ABCDEFGHIJ'),
                 'teams': [{'login': f'team{i:05}', 'nome': f'Team {i}', 'escola': 'Example University'} for i in range(args.teams)],
                 'score_freeze_time_seconds': 999999, 'penalty_seconds': 1200, 'time_seconds': 60}
        request('POST', event, state)
        request('POST', '/internal/contests/' + urllib.parse.quote(args.event, safe='') + '/' + urllib.parse.quote(args.contest, safe=''), {'name': args.contest, 'codes': ['team']})
        for start in range(0, args.runs, 500):
            batch = [{'id': i, 'team_login': f'team{i % args.teams:05}', 'prob': chr(65 + i % 10), 'time_seconds': 10 + i % 100, 'answer': 'Y' if i % 3 == 0 else 'N'} for i in range(start, min(start + 500, args.runs))]
            request('POST', event + '/runs', {'runs': batch})
            save_runs(batch)
        snapshot()
    elif args.mode == 'freeze':
        sent = time.time_ns() // 1_000_000
        request('PATCH', event, {'score_freeze_time_seconds': args.freeze_seconds})
        state, _ = snapshot()
        record_freeze(sent, state)
    elif args.mode == 'snapshot':
        snapshot()
    else:
        state, runs = snapshot()
        next_id = max((r['id'] for r in runs), default=-1) + 1
        with args.ledger.open('a') as ledger:
            def publish(batch):
                sent = time.time_ns() // 1_000_000
                request('POST', event + '/runs', {'runs': batch})
                save_runs(batch)
                for run in batch:
                    public = dict(run)
                    if public['time_seconds'] >= state['score_freeze_time_seconds']:
                        public['answer'] = '?'
                    ledger.write(json.dumps({'kind': 'run', 'sent_unix_ms': sent, 'message': public}) + '\n')
                ledger.flush()
            if args.mode == 'correct':
                batch = [dict(run, answer='N' if run['answer'] == 'Y' else 'Y') for run in runs[:2000]]
                publish(batch)
                snapshot()
            else:
                start = time.monotonic()
                initial_time = state['time_seconds']
                for second in range(args.seconds):
                    if second == args.freeze_at:
                        sent = time.time_ns() // 1_000_000
                        request('PATCH', event, {'score_freeze_time_seconds': args.freeze_seconds})
                        state = request('GET', event)
                        record_freeze(sent, state)
                    if second == args.correct_at:
                        publish([dict(run, answer='N' if run['answer'] == 'Y' else 'Y') for run in list(known_runs.values())[:2000]])
                    batch = [{'id': next_id + second * 10 + i, 'team_login': f'team{(second * 10 + i) % args.teams:05}', 'prob': chr(65 + i), 'time_seconds': 110 + second, 'answer': 'Y' if i % 3 == 0 else 'N'} for i in range(10)]
                    publish(batch)
                    current = initial_time + second + 1
                    sent = time.time_ns() // 1_000_000
                    request('PATCH', event + '/time', {'time_seconds': current})
                    ledger.write(json.dumps({'kind': 'timer', 'sent_unix_ms': sent, 'message': {'current_time_seconds': current, 'score_freeze_time_seconds': state['score_freeze_time_seconds']}}) + '\n')
                    ledger.flush()
                    snapshot()
                    time.sleep(max(0, start + second + 1 - time.monotonic()))


if __name__ == '__main__':
    main()
