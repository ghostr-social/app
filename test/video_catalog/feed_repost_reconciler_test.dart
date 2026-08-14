import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/repost_samples.dart';

void main() {
  test(
    'keeps an accepted repost until a relay observation acknowledges it',
    () {
      final reconciler = FeedRepostReconciler();
      final original = repostablePost();
      final accepted = original.withRepost(true);
      reconciler.accept(accepted, [original]);

      final stale = reconciler.reconcile([original], [accepted]).single;
      expect(stale.viewerHasReposted, isTrue);

      final observed = original.withRepost(
        true,
        observation: VideoRepostObservation.observed,
      );
      expect(reconciler.reconcile([observed], [stale]).single, observed);

      final removed = original.withRepost(
        false,
        observation: VideoRepostObservation.observed,
      );
      expect(
        reconciler.reconcile([removed], [observed]).single.viewerHasReposted,
        isFalse,
      );
    },
  );
}
