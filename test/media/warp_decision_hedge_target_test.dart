import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('hedge action resolves its primary action and alternate source', () {
    final evidence = WarpDecisionEvidence.parse(
      jsonEncode({
        'schema_version': 1,
        'decisions': {
          'records': [_hedgeRecord()],
        },
      }),
    );

    final selected = evidence.records.single.selected;
    expect(selected?.kind, 'hedge');
    expect(selected?.command, 'hedge');
    expect(selected?.targetActionId, 17);
    expect(selected?.sourceId, 'opaque-alternate');
  });
}

Map<String, Object?> _hedgeRecord() => {
  'sequence': 2,
  'chosen_action_id': 23,
  'eventual_outcome': {'status': 'succeeded', 'bytes': 4096, 'elapsed_ms': 8},
  'warp_decision': {
    'additional_request_slot_demanded': true,
    'selected': {
      'planner_action_id': 23,
      'post_id': 'future',
      'kind': {
        'kind': 'hedge',
        'primary_action_id': 17,
        'alternate_source_id': 'opaque-alternate',
      },
      'command': {
        'command': 'hedge',
        'primary_action_id': 17,
        'transfer': {'source_id': 'opaque-alternate'},
      },
    },
  },
};
