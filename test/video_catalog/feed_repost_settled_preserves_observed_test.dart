import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/repost_samples.dart';

void main() {
  test('late patient state cannot replace newer observed feed truth', () {
    final post = repostablePost();
    final current = post.withRepost(
      true,
      observation: VideoRepostObservation.observed,
    );
    final stale = post.withRepost(
      false,
      observation: VideoRepostObservation.observed,
    );

    final settled = FeedRepostReconciler().settled([stale], [current]);

    expect(settled.single.viewerHasReposted, isTrue);
  });
}
