import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('resync does not append another event for the same media', () {
    final original = samplePost(id: 'original');
    final duplicate = samplePost(id: 'duplicate').withMedia(original.media);
    final roster = FeedRoster([original]);

    final resynced = roster.resynced([original, duplicate]);

    expect(resynced.posts, [original]);
  });
}
