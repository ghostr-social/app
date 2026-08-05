import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/ffi_progressive_playback_gateway.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

void main() {
  test('resolves a proxied progressive source through the FFI', () async {
    FfiFocusItem? sent;
    final gateway = FfiProgressivePlaybackGateway(
      resolvePlaybackUrl: ({required FfiFocusItem item}) async {
        sent = item;
        return 'http://127.0.0.1:3210/video.mp4?id=${item.postId}';
      },
    );
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/clip.mp4'),
      'e' * 64,
    );

    final proxied = await gateway.resolve(media);

    expect(sent?.postId, 'e' * 64);
    expect(sent?.urls, ['https://media.test/clip.mp4']);
    expect(sent?.delivery, FfiMediaDelivery.progressive);
    expect(
      proxied.playbackUri.toString(),
      'http://127.0.0.1:3210/video.mp4?id=${'e' * 64}',
    );
  });
}
