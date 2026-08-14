import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';

import '../support/repost_samples.dart';

void main() {
  test('an unobserved refresh preserves current viewer repost state', () {
    final reconciler = FeedRepostReconciler();
    final original = repostablePost();
    final current = original.withRepost(true);

    final refreshed = reconciler.reconcile([original], [current]).single;

    expect(refreshed.viewerHasReposted, isTrue);
  });
}
