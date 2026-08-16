import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/repost_samples.dart';
import '../support/sample_data.dart';

void main() {
  test('blocking a reposter removes its held feed occurrence', () {
    final repost = repostedPost();
    final survivor = samplePost(id: 'survivor');

    final filtered = FeedRoster([
      repost,
      survivor,
    ]).withoutBlocked({repost.repost!.reposter.id});

    expect(filtered.posts, [survivor]);
  });
}
