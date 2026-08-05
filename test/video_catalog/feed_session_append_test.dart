import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_session.dart';

import '../support/sample_data.dart';

void main() {
  test('an older page that repeats what the viewer has adds nothing', () {
    final session = FeedSession();
    final roster = FeedRoster([samplePost(id: 'post-0')]);
    session.loaded(roster.posts);

    expect(session.appended(roster, [samplePost(id: 'post-0')]), isNull);

    final grown = session.appended(roster, [samplePost(id: 'older-0')]);

    expect(grown?.map((post) => post.id.value), ['post-0', 'older-0']);
    expect(session.held.map((post) => post.id.value), ['post-0', 'older-0']);
  });
}
