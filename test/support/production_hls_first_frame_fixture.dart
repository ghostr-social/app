import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import 'fake_hls_playback_gateway.dart';
import 'fake_remote_video_source.dart';
import 'fake_video_player_platform.dart';
import 'recording_player_preparation_feedback.dart';
import 'test_video_delivery.dart';
import 'video_player_surface_pump.dart';

final class ProductionHlsFirstFrameFixture {
  ProductionHlsFirstFrameFixture() {
    VideoPlayerPlatform.instance = platform;
    frames = NativeRenderedFirstFramePort(events: events.stream);
    telemetry = _telemetry(observationPosts, presentationPosts);
    playback = buildProductionVideoPlayback(
      testVideoDelivery(
        remoteSource: FakeRemoteVideoSource([]),
        hlsPlaybackGateway: gateway,
        playbackCapabilities: VideoPlaybackCapabilities.progressiveAndHls,
      ),
      playbackTelemetry: telemetry,
      playerPreparationFeedback: feedback,
      renderedFirstFrames: frames,
    );
  }

  final platform = FakeVideoPlayerPlatform();
  final events = StreamController<Object?>();
  final observationPosts = <String>[];
  final presentationPosts = <String>[];
  final feedback = RecordingPlayerPreparationFeedback();
  final gateway = FakeHlsPlaybackGateway();
  late final NativeRenderedFirstFramePort frames;
  late final FfiPlaybackTelemetryPort telemetry;
  late final VideoPlaybackPort playback;

  String get deliveryId => gateway.requests.single.deliveryId.value;
  String? get token =>
      platform.dataSources.single.httpHeaders[warpPlaybackAttemptHeader];

  Future<void> mount(WidgetTester tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: _media,
            videoId: PlaybackVideoId.parse('clip'),
            isActive: true,
          ),
        ),
      ),
    );
    gateway.completeNext();
    await tester.pump();
    await settleVideoPlayerTasks(tester);
    await telemetry.settled;
  }

  void emitNativeFrame() => events.add({'version': 1, 'attemptToken': token});

  Future<void> settlePresentation(WidgetTester tester) async {
    await tester.pump();
    await telemetry.settled;
  }

  Future<void> close() => frames.dispose().whenComplete(events.close);
}

FfiPlaybackTelemetryPort _telemetry(
  List<String> observations,
  List<String> presentations,
) {
  return FfiPlaybackTelemetryPort(
    reportPlayback: ({required input}) async => observations.add(input.postId),
    reportPresentation: ({required input}) async =>
        presentations.add(input.postId),
  );
}

final _media = VideoMediaSource.withCacheScope(
  VideoMediaSource.remote(
    'https://media.example/master.m3u8',
    delivery: VideoMediaDelivery.hls,
  ),
  'hls-post',
);
