import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_session.dart';

import '../support/sample_data.dart';

void main() {
  test('one batch cannot queue refreshed signatures of the same video', () {
    final first = samplePost(id: 'first').withMedia(
      VideoMediaSource.remote('https://cdn.example/clip.mp4?token=old'),
    );
    final republished = samplePost(id: 'republished').withMedia(
      VideoMediaSource.remote('https://cdn.example/clip.mp4?token=new'),
    );
    final fresh = samplePost(id: 'fresh');

    final roster = FeedSession().loaded([first, republished, fresh]);

    expect(roster.posts.map((post) => post.id.value), ['first', 'fresh']);
  });
}
