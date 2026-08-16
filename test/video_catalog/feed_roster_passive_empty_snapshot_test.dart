import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';

import '../support/sample_data.dart';

void main() {
  test('a passive empty snapshot keeps the admitted roster', () {
    final post = samplePost(id: 'a');
    final roster = FeedRoster([post]);

    expect(roster.resynced(const []).posts, [post]);
  });
}
