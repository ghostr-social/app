import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('material history retains selection-free capacity demand', () {
    final history = WarpDecisionEvidence.parse(
      jsonEncode({
        'schema_version': 1,
        'decisions': {
          'records': [_capacityDemandRecord()],
        },
      }),
    );

    expect(history.materialRecords.single.sequence, 1);
    expect(
      history.materialRecords.single.additionalRequestSlotDemanded,
      isTrue,
    );
  });
}

Map<String, Object?> _capacityDemandRecord() => {
  'sequence': 1,
  'chosen_action_id': null,
  'eventual_outcome': {'status': 'succeeded', 'bytes': 0, 'elapsed_ms': 0},
  'warp_decision': {'additional_request_slot_demanded': true, 'selected': null},
};
