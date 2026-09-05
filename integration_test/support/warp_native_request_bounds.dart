import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'warp_evidence_reader.dart';

Future<void> expectWarpRequestBounds(WarpEvidenceReader evidence) async {
  final history = await evidence.decisions();
  final counts = history.records
      .map((record) => record.requestOccupancy)
      .nonNulls;
  expect(
    counts,
    isNotEmpty,
    reason: 'Native broker occupancy must be recorded.',
  );
  var totalPeak = 0;
  var originPeak = 0;
  for (final count in counts) {
    expect(
      count.withinCoreBounds,
      isTrue,
      reason: 'active=${count.total} per_origin=${count.maximumPerOrigin}',
    );
    if (count.total > totalPeak) totalPeak = count.total;
    if (count.maximumPerOrigin > originPeak) {
      originPeak = count.maximumPerOrigin;
    }
  }
  expect(
    totalPeak,
    greaterThan(0),
    reason: 'The evidence must include active requests.',
  );
  debugPrint(
    'WARP_REQUEST_BOUNDS broker_peak=$totalPeak per_origin_peak=$originPeak',
  );
}
