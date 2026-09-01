import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('plan cursor starts immediately before retained evidence', () {
    expect(_page(513, 700).beforeOldestRetainedRevision, 512);
    expect(_page(0, 0).beforeOldestRetainedRevision, 0);
  });

  test('latest plan cursor skips the older retained backlog', () {
    expect(_page(513, 700).beforeLatestRetainedRevision, 699);
    expect(_page(0, 0).beforeLatestRetainedRevision, 0);
  });
}

WarpPlanPage _page(int oldest, int latest) => WarpPlanPage(
  oldestRetainedRevision: oldest,
  latestRetainedRevision: latest,
  cursorTruncated: false,
  hasMore: false,
  records: const [],
);
