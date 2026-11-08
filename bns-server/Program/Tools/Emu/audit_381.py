"""Audit saved oracle evidence and extract exact Rust mismatch outputs."""
import json
from pathlib import Path
from collections import Counter
from emu381 import VECTORS

root = Path(__file__).resolve().parent
vectors = json.loads(VECTORS.read_text())
cases = vectors['cases']
assert len(cases) == 62
for c in cases:
    assert any(call['address'] == c['source'] and call['sentinel']
               and call['instructions'] > 1 for call in c['calls'])
    if c['kind'].startswith('session_'):
        if c['result'] == 0:
            source = bytes.fromhex(c['input'])
            if c['kind'] == 'session_decrypt':
                source = source[2:]
            assert bytes.fromhex(c['output']) != source
        else:
            assert c['output'] == ''
            assert c['error_output'] != c['input']
text = (root / 'SESSION-RESULT.txt').read_text()
start = text.index('[\n')
report, _ = json.JSONDecoder().raw_decode(text[start:])
assert len(report) == 4
(root / 'SESSION-DIFFERENCES.json').write_text(json.dumps(report, indent=2) + '\n')
print('Case counts:', dict(Counter(c['kind'] for c in cases)))
print('Current stubs:', vectors['stub_counts'])
print('Reused RSA/DH stubs:', vectors.get('reused_rsa_stub_counts'))
for c in report:
    print(c['case'], 'original return', c['original_result'],
          'Rust Ok length', c['rust_output_length'])
    if c['rust_output_length'] <= 64:
        print('  Rust output:', c['rust_output'])
    if c['original_output']:
        print('  Original output:', c['original_output'])
print('Audit passed: all 62 cases have sentinel evidence; all 18 successful session outputs differ from their inputs.')
