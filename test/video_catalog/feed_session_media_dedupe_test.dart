import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_session.dart';

import '../support/sample_data.dart';

void main() {
  test('one load never queues the same video file twice', () {
    final session = FeedSession();
    final first = samplePost(id: 'first');
    final sameFileNewEvent = samplePost(id: 'republished').withMedia(
      VideoMediaSource.remote('https://example.com/video/first.mp4'),
    );
    final fresh = samplePost(id: 'fresh');

    final roster = session.loaded([first, sameFileNewEvent, fresh]);

    expect(roster.posts.map((post) => post.id.value), ['first', 'fresh']);
  });
}
