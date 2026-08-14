import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/repost_samples.dart';

void main() {
  test('late observed repost state projects onto the current row', () {
    final current = repostablePost();
    final observed = current.withRepost(
      true,
      observation: VideoRepostObservation.observed,
    );

    final settled = FeedRepostReconciler().settled([observed], [current]);

    expect(settled.single.viewerHasReposted, isTrue);
    expect(
      settled.single.repostContext.observation,
      VideoRepostObservation.observed,
    );
  });
}
