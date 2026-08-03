import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

import '../support/sample_data.dart';

void main() {
  test('replacing media keeps the post hashtags', () {
    final post = samplePost(hashtags: ['nostr', 'dance']);

    final replaced = post.withMedia(
      VideoMediaSource.remote('https://cdn.example/replacement.mp4'),
    );

    expect(replaced.hashtags, ['nostr', 'dance']);
    expect(replaced.media.remoteUrl, 'https://cdn.example/replacement.mp4');
  });
}
