import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_pagination.dart';

import '../support/sample_data.dart';

void main() {
  test('an older page never re-adds a video the session already plays', () {
    final playing = samplePost(id: 'playing');
    final sameFileNewEvent = samplePost(id: 'republished').withMedia(
      VideoMediaSource.remote('https://example.com/video/playing.mp4'),
    );
    final fresh = samplePost(id: 'fresh');

    final appended = FeedPagination.appendNew(
      [playing],
      [sameFileNewEvent, fresh],
    );

    expect(appended.map((post) => post.id.value), ['playing', 'fresh']);
  });
}
