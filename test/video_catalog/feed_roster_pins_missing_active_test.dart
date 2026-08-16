import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('passive resync pins an active post missing from the snapshot', () {
    final active = samplePost(id: 'removed');
    final next = samplePost(id: 'survivor');

    final refreshed = FeedRoster([
      active,
      next,
    ]).resynced([next], eligible: [next]);

    expect(refreshed.posts, [active, next]);
    expect(refreshed.active, active);
  });
}
