import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/ffi_focus_item_media_mapper.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

void main() {
  test('maps remote media onto the FFI focus item payload', () {
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.withExpectedSha256(
        VideoMediaSource.remote(
          'https://media.test/clip.mp4',
          fallbackUrls: const ['https://mirror.test/clip.mp4'],
          metadata: const VideoMediaMetadata(sizeBytes: 1234, durationMs: 5678),
        ),
        'a' * 64,
      ),
      'f' * 64,
    );

    final item = ffiFocusItemForMedia(media);

    expect(item.postId, 'f' * 64);
    expect(item.urls, [
      'https://media.test/clip.mp4',
      'https://mirror.test/clip.mp4',
    ]);
    expect(item.delivery, FfiMediaDelivery.progressive);
    expect(item.sha256, 'a' * 64);
    expect(item.sizeBytes, BigInt.from(1234));
    expect(item.durationMs, BigInt.from(5678));
  });

  test('maps HLS delivery and omits absent metadata', () {
    final media = VideoMediaSource.remote(
      'https://media.test/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    final item = ffiFocusItemForMedia(media);

    expect(item.delivery, FfiMediaDelivery.hls);
    expect(item.sha256, isNull);
    expect(item.sizeBytes, isNull);
    expect(item.durationMs, isNull);
  });

  test('refuses media without a remote source', () {
    expect(
      () => ffiFocusItemForMedia(VideoMediaSource.local('/videos/draft.mp4')),
      throwsArgumentError,
    );
  });
}
