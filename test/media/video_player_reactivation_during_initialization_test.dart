import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets(
    'reactivation supersedes an initialization that never completed',
    (tester) async {
      final platform = AuditedVideoPlayerPlatform(autoInitialize: false);
      VideoPlayerPlatform.instance = platform;
      final port = VideoPlayerPlaybackPort();
      final media = VideoMediaSource.local('/cache/video.mp4');

      await _show(tester, port, media, true);
      expect(platform.createdCount, 1);
      await _show(tester, port, media, false);
      await _show(tester, port, media, true);
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      await tester.pump();

      expect(platform.createdCount, 2);
      platform.initialize(1);
      await tester.pump();
      expect(find.byType(Texture), findsOneWidget);
    },
  );
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoMediaSource media,
  bool active,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(media: media, isActive: active),
      ),
    ),
  );
  await tester.pump();
}
