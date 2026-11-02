import importlib.util
import io
import json
import pathlib
import unittest

spec = importlib.util.spec_from_file_location('latency', pathlib.Path(__file__).parents[1] / 'analyze_viewer_latency.py')
latency = importlib.util.module_from_spec(spec)
spec.loader.exec_module(latency)


class LatencyTests(unittest.TestCase):
    def test_matches_normalized_messages_and_reports_clock_skew(self):
        ledger = io.StringIO(json.dumps({'kind': 'run', 'sent_unix_ms': 100, 'message': {'id': 1, 'answer': 'Y'}}) + '\n')
        trace = io.StringIO('\n'.join(json.dumps({'kind': 'run', 'received_unix_ms': received, 'message': {'answer': 'Y', 'id': 1}}) for received in [90, 110, 120]))
        result = latency.analyze(ledger, trace)
        self.assertEqual(result['p99_ms'], 20)
        self.assertEqual(result['samples'], 2)
        self.assertEqual(result['negative_clock_samples'], 1)

    def test_replay_is_not_counted_as_live_latency(self):
        trace = io.StringIO(json.dumps({'kind': 'run', 'received_unix_ms': 100, 'message': {'id': 1}}))
        result = latency.analyze(io.StringIO(''), trace)
        self.assertEqual(result['unmatched_replay_messages'], 1)
        self.assertIsNone(result['p99_ms'])
