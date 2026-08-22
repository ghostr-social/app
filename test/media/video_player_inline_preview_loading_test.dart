import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/widgets/inline_blurhash_preview.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('shows inline preview until initialized video replaces it', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform(autoInitialize: false);
    VideoPlayerPlatform.instance = platform;
    final media = VideoMediaSource.local('/cache/clip.mp4');
    final preview = InlineBlurHash.parse('000000');

    await tester.pumpWidget(
      MaterialApp(
        home: VideoPlayerPlaybackPort().buildSurface(
          VideoPlaybackSurfaceRequest(
            media: media,
            isActive: true,
            preview: preview,
          ),
        ),
      ),
    );

    expect(find.byType(InlineBlurHashPreview), findsOneWidget);
    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    await tester.pump();
    platform.initialize(0);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byType(VideoPlayer), findsOneWidget);
    expect(find.byType(InlineBlurHashPreview), findsNothing);
  });
}
