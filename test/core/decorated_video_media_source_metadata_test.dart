import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  const metadata = VideoMediaMetadata(sizeBytes: 1234, durationMs: 5000);

  test('sha256 and cache-scope decorators keep remote metadata visible', () {
    var source = VideoMediaSource.remote(
      'https://example.com/video.mp4',
      metadata: metadata,
    );
    source = VideoMediaSource.withExpectedSha256(source, 'a' * 64);
    source = VideoMediaSource.withCacheScope(source, 'scope-1');

    expect(source.mediaMetadata, metadata);
  });

  test('local sources report empty media metadata', () {
    final source = VideoMediaSource.local('/tmp/clip.mp4');

    expect(source.mediaMetadata, VideoMediaMetadata.none);
  });
}
