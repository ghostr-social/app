import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('material history retains transfer and its later cancellation', () {
    final records = <Map<String, Object?>>[
      _transferRecord(),
      ...List.generate(25, (index) => _noopRecord(index + 2)),
      _zeroByteCancellation(),
      _cancelRecord(),
    ];
    final history = WarpDecisionEvidence.parse(
      jsonEncode({
        'schema_version': 1,
        'decisions': {'records': records},
      }),
    );

    expect(
      history.materialRecords.map((record) => record.sequence),
      orderedEquals([1, 27, 28]),
    );
    expect(history.materialRecords.last.selected?.targetActionId, 17);
  });
}

Map<String, Object?> _transferRecord() => {
  'sequence': 1,
  'chosen_action_id': 17,
  'eventual_outcome': {'status': 'cancelled', 'bytes': 16384, 'elapsed_ms': 20},
  'warp_decision': {
    'additional_request_slot_demanded': false,
    'selected': {
      'planner_action_id': 5,
      'post_id': 'future',
      'kind': {'kind': 'fetch_range', 'bytes_start': 0, 'bytes_end': 262144},
      'command': {
        'command': 'transfer',
        'transfer': {'source_id': 'opaque-source'},
      },
    },
  },
  'executed_request': {
    'post_id': 'future',
    'source_id': 'opaque-source',
    'request': {
      'request': 'fetch_range',
      'bytes_start': 0,
      'bytes_end': 262144,
    },
    'resources': <String, Object?>{},
  },
};

Map<String, Object?> _noopRecord(int sequence) => {
  'sequence': sequence,
  'chosen_action_id': null,
  'eventual_outcome': {'status': 'succeeded', 'bytes': 0, 'elapsed_ms': 0},
};

Map<String, Object?> _zeroByteCancellation() => {
  'sequence': 27,
  'chosen_action_id': null,
  'eventual_outcome': {'status': 'cancelled', 'bytes': 0, 'elapsed_ms': 4},
};

Map<String, Object?> _cancelRecord() => {
  'sequence': 28,
  'chosen_action_id': null,
  'eventual_outcome': {'status': 'succeeded', 'bytes': 0, 'elapsed_ms': 0},
  'warp_decision': {
    'additional_request_slot_demanded': false,
    'selected': {
      'planner_action_id': 9,
      'post_id': 'future',
      'kind': {'kind': 'cancel', 'action_id': 17},
      'command': {'command': 'cancel', 'action_id': 17},
    },
  },
};
