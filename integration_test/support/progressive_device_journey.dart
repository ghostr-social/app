import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_progressive_playback_gateway.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_post.dart';
import 'progressive_device_resources.dart';
import 'progressive_device_telemetry.dart';
import 'progressive_device_wait.dart';
import 'progressive_playback_admissions.dart';

part 'progressive_device_journey_playback.dart';
part 'progressive_device_journey_metrics.dart';

final class ProgressiveDeviceJourney {
  ProgressiveDeviceJourney._({
    required ProgressiveDeviceResources resources,
    required this.posts,
    required ProgressiveDeviceTelemetry telemetry,
    required ProgressivePlaybackAdmissionProbe admissions,
  }) : _resources = resources,
       _telemetry = telemetry,
       _admissions = admissions,
       _playback = GatewayVideoPlaybackPort(
         delegate: VideoPlayerPlaybackPort(
           telemetry: telemetry,
           renderedFirstFrames: NativeRenderedFirstFramePort.production(),
         ),
         gateway: const FfiProgressivePlaybackGateway(),
       );

  static Future<ProgressiveDeviceJourney> start() async {
    final resources = await ProgressiveDeviceResources.start();
    try {
      final settings = AppSettings.defaults()
          .withRelays(const [])
          .withSearchRelays(const []);
      final started = await FfiVideoGateway().start(
        settings,
        resources.cachePath,
        deviceIntegrationOrigin: resources.origin.origin,
      );
      if (started is VideoGatewayFailed) throw StateError(started.message);
      final admissions = await ProgressivePlaybackAdmissionProbe.capture();
      return ProgressiveDeviceJourney._(
        resources: resources,
        posts: _posts(resources.origin),
        telemetry: ProgressiveDeviceTelemetry(),
        admissions: admissions,
      );
    } on Object {
      await resources.close();
      rethrow;
    }
  }

  final ProgressiveDeviceResources _resources;
  final List<VideoPost> posts;
  final ProgressiveDeviceTelemetry _telemetry;
  final ProgressivePlaybackAdmissionProbe _admissions;
  final VideoPlaybackPort _playback;
  final _focus = FfiFeedFocusPort();
  PlaybackFocus? _currentFocus;
  PlaybackFocus? _nextFocus;

  ProgressiveDeviceOrigin get origin => _resources.origin;

  Future<void> focusCurrentAndNext() async {
    _focus.focusChanged(FeedFocus.around(posts: posts, activeIndex: 0));
    await Future<void>.delayed(Duration.zero);
  }

  /// The native engine is process-scoped; this closes journey-owned resources.
  Future<void> close() => _resources.close();
}

List<VideoPost> _posts(ProgressiveDeviceOrigin origin) => [
  progressiveDevicePost(
    socialId: 'social-current',
    deliveryId: 'a' * 64,
    origin: origin.urlFor('current'),
  ),
  progressiveDevicePost(
    socialId: 'social-next',
    deliveryId: 'b' * 64,
    origin: origin.urlFor('next'),
  ),
];
