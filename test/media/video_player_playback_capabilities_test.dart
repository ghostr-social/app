import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_capabilities.dart';

void main() {
  final hls = VideoMediaSource.remote(
    'https://media.example/live.m3u8',
    delivery: VideoMediaDelivery.hls,
  );
  final progressive = VideoMediaSource.remote(
    'https://media.example/video.mp4',
    delivery: VideoMediaDelivery.progressive,
  );
  final local = VideoMediaSource.local('/videos/draft.mp4');

  test('advertises secure native HLS only on registered player backends', () {
    for (final platform in [
      TargetPlatform.android,
      TargetPlatform.iOS,
      TargetPlatform.macOS,
    ]) {
      final capabilities = videoPlayerPlaybackCapabilities(
        platform: platform,
        isWeb: false,
      );
      for (final media in [hls, progressive, local]) {
        expect(capabilities.supports(media), isTrue);
      }
    }
  });

  test('keeps unsupported native and web builds fail closed', () {
    for (final platform in [
      TargetPlatform.linux,
      TargetPlatform.windows,
      TargetPlatform.fuchsia,
    ]) {
      final capabilities = videoPlayerPlaybackCapabilities(
        platform: platform,
        isWeb: false,
      );
      for (final media in [hls, progressive, local]) {
        expect(capabilities.supports(media), isFalse);
      }
    }
    final web = videoPlayerPlaybackCapabilities(
      platform: TargetPlatform.android,
      isWeb: true,
    );
    for (final media in [hls, progressive, local]) {
      expect(web.supports(media), isFalse);
    }
  });
}
