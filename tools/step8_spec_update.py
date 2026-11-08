"""Validate step-8 DB metadata without changing files or using a network.

Run from the project root with Python isolated mode. This helper was not run in
step 8 because Python execution required approval. jq validated the JSON instead.
"""
import json
from pathlib import Path


def main():
    root = Path(__file__).resolve().parents[1]
    document = json.loads((root / 'spec/lobbyd/db-binding.json').read_text())
    calls = document['calls']
    assert len(calls) == 88
    maps = document['caller_error_maps_step8']
    checked = maps['fresh_login_D11_16']['procedures']
    assert len(checked) == len(set(checked)) == 13
    assert set(checked) <= {call['procedure'] for call in calls}
    assert maps['default_for_all_88_calls'].startswith('❓')
    print('PASS: 88 contracts; 13 fresh-login failure maps; other paths stay open.')


if __name__ == '__main__':
    main()
