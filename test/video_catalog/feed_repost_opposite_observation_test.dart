import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/repost_samples.dart';

void main() {
  test(
    'authoritative opposite state supersedes an unacknowledged mutation',
    () {
      final reconciler = FeedRepostReconciler();
      final original = repostablePost();
      final accepted = original.withRepost(true);
      reconciler.accept(accepted, [original]);
      final removed = original.withRepost(
        false,
        observation: VideoRepostObservation.observed,
      );

      final reconciled = reconciler.reconcile([removed], [accepted]).single;

      expect(reconciled.viewerHasReposted, isFalse);
      expect(reconciler.reconcile([removed], [reconciled]).single, removed);
    },
  );
}
