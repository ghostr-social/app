import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'device_playback_testbed.dart';
import 'device_video_scenario.dart';

final class DeviceHlsReactivationJourney {
  DeviceHlsReactivationJourney._(
    this.testbed,
    this.videoId,
    this.media,
    this.authority,
  );

  static Future<DeviceHlsReactivationJourney> start() async {
    final testbed = await DevicePlaybackTestbed.start(
      DeviceVideoScenario.contract,
    );
    const rawVideoId = 'hls-reactivation';
    final media = ProxiedHlsVideoMediaSource(
      testbed.server.playbackUri(rawVideoId).toString(),
    );
    final authority = HlsPlaybackAuthority(
      deliveryId: PlaybackDeliveryId.parse(
        testbed.server.deliveryIdFor(rawVideoId),
      ),
      representationId: VideoRepresentationId.parse('a' * 64),
      assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
    );
    return DeviceHlsReactivationJourney._(
      testbed,
      PlaybackVideoId.parse(rawVideoId),
      media,
      authority,
    );
  }

  final DevicePlaybackTestbed testbed;
  final PlaybackVideoId videoId;
  final ProxiedHlsVideoMediaSource media;
  final HlsPlaybackAuthority authority;
  final verified = <HlsPlaybackAuthority>[];
  final revoked = <HlsPlaybackAuthority>[];
  bool _shellMounted = false;

  Future<PlaybackFocus> show(
    WidgetTester tester, {
    required bool isActive,
  }) async {
    await _ensureShell(tester);
    final focus = testbed.probe.markFocus(videoId);
    await tester.pumpWidget(
      MaterialApp(home: SizedBox.expand(child: _surface(isActive))),
    );
    return focus;
  }

  Future<void> waitForFrame(WidgetTester tester, PlaybackFocus focus) {
    return testbed.waitUntil(
      tester,
      () => testbed.probe.firstFrameLatency(focus) != null,
    );
  }

  Future<void> waitForRevocation(WidgetTester tester) async {
    final watch = Stopwatch()..start();
    while (revoked.isEmpty && watch.elapsed < const Duration(seconds: 6)) {
      await tester.pump(const Duration(milliseconds: 50));
      await Future<void>.delayed(const Duration(milliseconds: 20));
    }
  }

  Future<void> close() => testbed.close();

  Future<void> _ensureShell(WidgetTester tester) async {
    if (_shellMounted) return;
    await tester.pumpWidget(const MaterialApp(home: SizedBox.expand()));
    _shellMounted = true;
  }

  Widget _surface(bool isActive) {
    return testbed.playback.buildSurface(
      VideoPlaybackSurfaceRequest(
        media: media,
        videoId: videoId,
        isActive: isActive,
        playbackDeliveryId: authority.deliveryId,
        hlsAuthority: authority,
        onHlsFirstFrameRendered: verified.add,
        onHlsDecodedReadinessRevoked: revoked.add,
      ),
    );
  }
}
