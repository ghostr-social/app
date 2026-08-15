import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('ordinary resync drops an active post missing from the raw feed', () {
    final active = samplePost(id: 'removed');
    final next = samplePost(id: 'survivor');

    final refreshed = FeedRoster([
      active,
      next,
    ]).resynced([next], eligible: [next], retainWatched: false);

    expect(refreshed.posts, [next]);
    expect(refreshed.active, next);
  });
}
