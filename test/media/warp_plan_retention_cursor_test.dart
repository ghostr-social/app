import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('plan cursor starts immediately before retained evidence', () {
    expect(_page(513).beforeOldestRetainedRevision, 512);
    expect(_page(0).beforeOldestRetainedRevision, 0);
  });
}

WarpPlanPage _page(int oldest) => WarpPlanPage(
  oldestRetainedRevision: oldest,
  latestRetainedRevision: oldest,
  cursorTruncated: false,
  hasMore: false,
  records: const [],
);
