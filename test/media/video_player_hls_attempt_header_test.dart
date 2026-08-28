import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('HLS correlation rejects progressive ranking authority', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final frames = NativeRenderedFirstFramePort(
      events: const Stream<Object?>.empty(),
      tokenFactory: () => RenderedFirstFrameAttemptToken.parse(_token),
    );
    addTearDown(frames.dispose);
    final feedback = RecordingPlayerPreparationFeedback();
    final port = VideoPlayerPlaybackPort(
      preparationFeedback: feedback,
      renderedFirstFrames: frames,
    );
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedHlsVideoMediaSource(_playbackUrl),
        isActive: false,
        authority: testPlaybackAuthority(postId: _sessionId),
      ),
    );

    expect(platform.dataSources.single.httpHeaders, {
      warpPlaybackAttemptHeader: isNotEmpty,
    });
    expect(feedback.events, isEmpty);
  });
}

const _sessionId =
    '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const _playbackUrl = 'http://127.0.0.1:3210/hls/$_sessionId/index.m3u8';
const _token = 'abcdefghijklmnopqrstuA';
