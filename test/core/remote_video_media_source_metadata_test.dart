import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('remote source carries imeta size and duration metadata', () {
    final source = VideoMediaSource.remote(
      'https://example.com/video.mp4',
      metadata: const VideoMediaMetadata(
        sizeBytes: 9000000,
        durationMs: 42000,
      ),
    ) as RemoteVideoMediaSource;

    expect(source.sizeBytes, 9000000);
    expect(source.durationMs, 42000);
    expect(
      source.mediaMetadata,
      const VideoMediaMetadata(sizeBytes: 9000000, durationMs: 42000),
    );
  });

  test('remote source defaults to empty metadata', () {
    final source = VideoMediaSource.remote('https://example.com/video.mp4')
        as RemoteVideoMediaSource;

    expect(source.sizeBytes, isNull);
    expect(source.durationMs, isNull);
    expect(source.mediaMetadata, VideoMediaMetadata.none);
  });
}
