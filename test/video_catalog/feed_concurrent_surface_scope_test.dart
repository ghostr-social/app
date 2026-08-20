import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_progressive_playback_gateway.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/feed_preparation_video_player_platform.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('two mounted feeds keep separate exact playback surfaces', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final playback = GatewayVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(),
      gateway: FakeProgressivePlaybackGateway(
        immediatePlaybackUrl: fakeProgressivePlaybackUrl,
      ),
    );
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/post-1.mp4'),
      'post-1',
    );
    final post = samplePost().withMedia(media);

    await tester.pumpWidget(_feeds(playback, post, ['a', 'b']));
    await tester.pumpAndSettle();

    expect(tester.takeException(), isNull);
    expect(platform.createdCount, 2);
  });

  testWidgets('disposed feed scope can remount the same exact video', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final playback = GatewayVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(),
      gateway: FakeProgressivePlaybackGateway(
        immediatePlaybackUrl: fakeProgressivePlaybackUrl,
      ),
    );
    final post = samplePost().withMedia(
      VideoMediaSource.withCacheScope(
        VideoMediaSource.remote('https://media.test/post-1.mp4'),
        'post-1',
      ),
    );

    await tester.pumpWidget(_feeds(playback, post, ['a', 'b']));
    await tester.pumpAndSettle();
    await tester.pumpWidget(_feeds(playback, post, ['b']));
    await tester.pumpAndSettle();
    await tester.pumpWidget(_feeds(playback, post, ['a', 'b']));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(tester.takeException(), isNull);
    expect(platform.createdCount, 3);
    expect(platform.playerCount, 2);
  });
}

Widget _feeds(VideoPlaybackPort port, VideoPost post, List<String> ids) {
  return Stack(
    textDirection: TextDirection.ltr,
    children: ids
        .map(
          (id) => KeyedSubtree(
            key: ValueKey(id),
            child: feedScreenHarness(
              FakeVideoCatalogRepository(forYouFeed: [post]),
              options: FeedScreenHarnessOptions(playbackPort: port),
            ),
          ),
        )
        .toList(),
  );
}
