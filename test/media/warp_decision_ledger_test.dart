import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_decision_ledger.dart';
import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('ledger keeps evicted records and adopts resolved outcomes', () {
    final ledger = WarpDecisionLedger();
    ledger.absorb(WarpDecisionEvidence.parse(_firstSample).records);
    ledger.absorb(WarpDecisionEvidence.parse(_secondSample).records);

    final records = ledger.records;
    expect(records.map((record) => record.sequence), [3, 4, 5]);
    expect(records[0].outcome.status, 'pending');
    expect(records[1].outcome.status, 'failed');
    expect(records[1].outcome.failureClass, 'Transient');
  });

  test('ledger starts empty', () {
    expect(WarpDecisionLedger().records, isEmpty);
  });
}

String _history(String records) =>
    '{"schema_version":1,"decisions":{"records":[$records]}}';

const _pending = '"eventual_outcome":{"status":"pending"}';

final _firstSample = _history(
  '{"sequence":3,"chosen_action_id":null,$_pending},'
  '{"sequence":4,"chosen_action_id":17,$_pending}',
);

final _secondSample = _history(
  '{"sequence":4,"chosen_action_id":17,"eventual_outcome":'
  '{"status":"failed","class":"Transient","bytes":0}},'
  '{"sequence":5,"chosen_action_id":null,$_pending}',
);
