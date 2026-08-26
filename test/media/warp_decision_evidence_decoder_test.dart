import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('WARP decision decoder correlates selected and executed range', () {
    final history = WarpDecisionEvidence.parse(_decisionJson);
    final decision = history.records.single;

    expect(decision.sequence, 4);
    expect(decision.chosenActionId, 17);
    expect(decision.selected?.plannerActionId, 5);
    expect(decision.executed?.sourceId, 'opaque-source');
    expect(decision.executed?.start, 65536);
    expect(decision.executed?.end, 131072);
    expect(decision.outcome.status, 'succeeded');
    expect(decision.outcome.bytes, 65536);
  });
}

const _decisionJson = r'''
{"schema_version":1,"decisions":{"records":[{
  "sequence":4,"chosen_action_id":17,
  "eventual_outcome":{"status":"succeeded","bytes":65536,"elapsed_ms":120},
  "warp_decision":{"selected":{"planner_action_id":5,"post_id":"opaque-post","kind":{"kind":"fetch_range","bytes_start":65536,"bytes_end":131072},"command":{"command":"transfer","transfer":{"post_id":"opaque-post","source_id":"opaque-source","request":{"request":"fetch_range","bytes_start":65536,"bytes_end":131072,"promotion":null},"expected_playable_gain_ms":500,"utility":{"view_probability_bits":0,"additional_playable_ms":500,"expected_delivery_ms":100,"score_bits":0},"authority":"transition","commitment_until_ms":200,"reason":"next_startability"}},"resources":{"network_bytes":65536,"storage_bytes":65536,"cpu_ms":0,"requests":1},"dependencies":[],"ready_playback_ms":500,"static_score_micros":1}},
  "executed_request":{"post_id":"opaque-post","source_id":"opaque-source","request":{"request":"fetch_range","bytes_start":65536,"bytes_end":131072,"promotion":null},"resources":{"network_bytes":65536,"storage_bytes":65536,"cpu_ms":0,"requests":1}}
}]}}
''';
