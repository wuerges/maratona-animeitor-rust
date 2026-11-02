#!/usr/bin/env python3
"""Join publisher and viewer JSONL traces; timestamps require synchronized hosts."""
import argparse
import bisect
import json
import math


def key(record):
    return record['kind'], json.dumps(record['message'], sort_keys=True, separators=(',', ':'))


def analyze(ledger, trace):
    sent = {}
    for line in ledger:
        record = json.loads(line)
        sent.setdefault(key(record), []).append(record['sent_unix_ms'])
    for timestamps in sent.values():
        timestamps.sort()
    values = []
    unmatched = 0
    skew = 0
    for line in trace:
        record = json.loads(line)
        published = sent.get(key(record))
        if published is None:
            unmatched += 1
            continue
        index = bisect.bisect_right(published, record['received_unix_ms']) - 1
        elapsed = record['received_unix_ms'] - published[max(0, index)]
        if elapsed < 0:
            skew += 1
        else:
            values.append(elapsed)
    values.sort()
    return {'samples': len(values), 'unmatched_replay_messages': unmatched, 'negative_clock_samples': skew,
            'p99_ms': values[max(0, math.ceil(len(values) * .99) - 1)] if values else None,
            'max_ms': values[-1] if values else None,
            'measurement': 'request-send to sampled browser-style receive; includes mutation processing'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ledger', required=True)
    parser.add_argument('--trace', required=True)
    parser.add_argument('--requests')
    args = parser.parse_args()
    with open(args.ledger) as ledger, open(args.trace) as trace:
        result = analyze(ledger, trace)
    if args.requests:
        with open(args.requests) as requests:
            durations = sorted(r['duration_ms'] for line in requests if (r := json.loads(line))['method'] in {'POST', 'PUT', 'PATCH', 'DELETE'})
        result['mutation_samples'] = len(durations)
        result['mutation_p99_ms'] = durations[max(0, math.ceil(len(durations) * .99) - 1)] if durations else None
    print(json.dumps(result, indent=2))
    if not result['samples'] or result['negative_clock_samples']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
