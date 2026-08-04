import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('exposes HLS only with both gateway and platform capabilities',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-hls-policy-');
    addTearDown(() => root.delete(recursive: true));
    final progressive = samplePost(id: 'progressive');
    final hls = samplePost(id: 'hls').withMedia(VideoMediaSource.remote(
      'https://media.example/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));
    final hlsGateway = FakeHlsPlaybackGateway();
    final delivery = await _build(
      root,
      [hls, progressive],
      hlsGateway,
      VideoPlaybackCapabilities.progressiveAndHls,
    );

    final posts = await delivery.remoteSource.loadRemoteFeed();

    expect(posts, [hls, progressive]);
    expect(delivery.hlsPlaybackGateway, same(hlsGateway));
  });

  test('keeps HLS hidden when the platform has no player backend', () async {
    final root = await Directory.systemTemp.createTemp('ghostr-hls-policy-');
    addTearDown(() => root.delete(recursive: true));
    final progressive = samplePost(id: 'progressive');
    final hls = samplePost(id: 'hls').withMedia(VideoMediaSource.remote(
      'https://media.example/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));
    final delivery = await _build(
      root,
      [hls, progressive],
      FakeHlsPlaybackGateway(),
      VideoPlaybackCapabilities.progressiveOnly,
    );

    expect(await delivery.remoteSource.loadRemoteFeed(), [progressive]);
    expect(delivery.hlsPlaybackGateway, isNull);
  });
}

Future<ProductionVideoDelivery> _build(
  Directory root,
  List<VideoPost> posts,
  FakeHlsPlaybackGateway hlsGateway,
  VideoPlaybackCapabilities capabilities,
) {
  return buildProductionVideoDelivery(
    AppSettings.defaults(),
    ProductionVideoDeliveryEnvironment(
      canonicalSource: FakeRemoteVideoSource(posts),
      supportDirectoryProvider: () async => root,
      gateway: startedVideoGateway(),
      hlsPlaybackGateway: hlsGateway,
      playbackCapabilities: capabilities,
    ),
  );
}
