import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/sample_data.dart';

void main() {
  test('round trips cached media with its remote fallback', () {
    final post = samplePost().withMedia(
      VideoMediaSource.withCacheScope(
        VideoMediaSource.withExpectedSha256(
          VideoMediaSource.cached(
            '/cache/video.mp4',
            remoteUrl: 'https://media.example/video.mp4',
            fallbackUrls: ['https://mirror.example/video.mp4'],
            delivery: VideoMediaDelivery.hls,
          ),
          'e3b0c44298fc1c149afbf4c8996fb924'
          '27ae41e4649b934ca495991b7852b855',
        ),
        'event-revision-1',
      ),
    );
    const mapper = VideoPostStorageMapper();

    final restored = mapper.fromMap(mapper.toMap(post));

    expect(post.media.debugLabel, '/cache/video.mp4');
    expect(restored.media.debugLabel, '/cache/video.mp4');
    expect(restored.media.localPath, '/cache/video.mp4');
    expect(restored.media.remoteUrls, [
      'https://media.example/video.mp4',
      'https://mirror.example/video.mp4',
    ]);
    expect(restored.media.remoteDelivery, VideoMediaDelivery.hls);
    expect(
      restored.media.expectedSha256?.value,
      'e3b0c44298fc1c149afbf4c8996fb924'
      '27ae41e4649b934ca495991b7852b855',
    );
    expect(restored.media.cacheScope?.value, 'event-revision-1');
  });
}
