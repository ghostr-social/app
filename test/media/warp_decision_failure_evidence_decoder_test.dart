import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('WARP decision decoder preserves immediate failure detail', () {
    final history = WarpDecisionEvidence.parse(_decisionJson);

    expect(
      history.records.first.outcome.failureClass,
      'origin_admission_blocked',
    );
    expect(history.records.first.outcome.claimRefusal, isNull);
    expect(history.records.last.outcome.failureClass, isNull);
    expect(history.records.last.outcome.claimRefusal, 'pool_at_capacity');
  });
}

const _decisionJson = r'''
{"schema_version":1,"decisions":{"records":[
  {"sequence":1,"chosen_action_id":null,
   "eventual_outcome":{"status":"failed","class":"origin_admission_blocked","elapsed_ms":0},
   "warp_decision":null,"executed_request":null},
  {"sequence":2,"chosen_action_id":null,
   "eventual_outcome":{"status":"claim_refused","reason":"pool_at_capacity"},
   "warp_decision":null,"executed_request":null}
]}}
''';
