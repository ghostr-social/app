import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/ffi_progressive_playback_gateway.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';

void main() {
  testWidgets(
    'gateway owns mirrors while the failed player reconnects locally',
    (tester) async {
      final platform = RecoveringVideoPlayerPlatform(initializationFailures: 1);
      VideoPlayerPlatform.instance = platform;
      final requests = <FfiFocusItem>[];
      final gateway = FfiProgressivePlaybackGateway(
        resolvePlaybackUrl: ({required item}) async {
          requests.add(item);
          final capability = requests.length == 1
              ? 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
              : 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
          return 'http://127.0.0.1:3210/video.mp4?'
              'id=${item.postId}&cap=$capability';
        },
      );
      final playback = GatewayVideoPlaybackPort(
        delegate: VideoPlayerPlaybackPort(
          recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
        ),
        gateway: gateway,
      );
      final media = VideoMediaSource.remote(
        'https://primary.test/video.mp4',
        fallbackUrls: const ['https://mirror.test/video.mp4'],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: playback.buildSurface(
            VideoPlaybackSurfaceRequest(media: media, isActive: true),
          ),
        ),
      );
      await tester.pump();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pump();

      expect(requests, hasLength(2));
      expect(requests.first.urls, [
        'https://primary.test/video.mp4',
        'https://mirror.test/video.mp4',
      ]);
      expect(requests.last.urls, requests.first.urls);
      expect(platform.dataSources, hasLength(2));
      expect(
        platform.dataSources.first.uri,
        isNot(platform.dataSources.last.uri),
      );
      expect(find.byType(VideoPlayer), findsOneWidget);
      expect(find.text('Video unavailable'), findsNothing);
    },
  );
}
