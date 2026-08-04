import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_session.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';

import '../support/sample_data.dart';

void main() {
  test('an older page that repeats what the viewer has adds nothing', () {
    final session = FeedSession();
    final feed = FeedLoaded(FeedKind.forYou, [samplePost(id: 'post-0')]);
    session.loaded(feed.posts);

    expect(session.appended(feed, [samplePost(id: 'post-0')]), isNull);

    final grown = session.appended(feed, [samplePost(id: 'older-0')]);

    expect(grown?.map((post) => post.id.value), ['post-0', 'older-0']);
    expect(session.held.map((post) => post.id.value), ['post-0', 'older-0']);
  });
}
