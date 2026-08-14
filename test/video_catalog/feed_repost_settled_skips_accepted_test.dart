import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/repost_samples.dart';

void main() {
  test('late hydration cannot overwrite a locally accepted repost', () {
    final original = repostablePost();
    final accepted = original.withRepost(
      true,
      observation: VideoRepostObservation.observed,
    );
    final reconciler = FeedRepostReconciler();
    final current = reconciler.accept(accepted, [original]);
    final stale = original.withRepost(
      false,
      observation: VideoRepostObservation.observed,
    );

    final settled = reconciler.settled([stale], current);

    expect(settled.single.viewerHasReposted, isTrue);
  });
}
