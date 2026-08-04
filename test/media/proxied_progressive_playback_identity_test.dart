import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('keys playback surfaces by the proxied progressive stream', () {
    final first = ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:3210/video.mp4?id=post-1',
    );
    final second = ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:3210/video.mp4?id=post-2',
    );
    final firstAgain = ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:3210/video.mp4?id=post-1',
    );

    expect(
      first.inventoryPlaybackIdentity,
      isNot(second.inventoryPlaybackIdentity),
    );
    expect(
      first.inventoryPlaybackIdentity,
      firstAgain.inventoryPlaybackIdentity,
    );
  });
}
