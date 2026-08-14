import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';

import '../support/repost_samples.dart';

void main() {
  test('repost projection keeps padded coordinates distinct', () {
    final padded = repostablePublishedPost(
      kind: 34236,
      identifier: 'clip',
      publishedIdentifier: ' clip ',
    );
    final trimmed = repostablePublishedPost(
      kind: 34236,
      identifier: 'clip',
      publishedIdentifier: 'clip',
    );

    final projected = FeedRepostReconciler().project(padded.withRepost(true), [
      padded,
      trimmed,
    ]);

    expect(projected.first.viewerHasReposted, isTrue);
    expect(projected.last.viewerHasReposted, isFalse);
  });
}
