import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps progressive delivery when native cache preparation fails',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-native-fail-');
    addTearDown(() => root.delete(recursive: true));
    await File('${root.path}/native_video_inventory').writeAsString('occupied');
    var gatewayInitializations = 0;
    final canonical = samplePost(caption: 'Canonical progressive video');

    final delivery = await buildProductionVideoDelivery(
      AppSettings.defaults(),
      ProductionVideoDeliveryEnvironment(
        canonicalSource: FakeRemoteVideoSource([canonical]),
        supportDirectoryProvider: () async => root,
        gateway: FfiVideoGateway(
          initialize: () async {
            gatewayInitializations += 1;
          },
          startEngine: ({
            required String cacheDirectory,
            required String relayUrls,
            required String dataUsage,
            required BigInt maxStorageBytes,
          }) async =>
              '127.0.0.1:3000',
        ),
        playbackCapabilities: VideoPlaybackCapabilities.progressiveAndHls,
      ),
    );

    expect(await delivery.remoteSource.loadRemoteFeed(), [canonical]);
    expect(delivery.playbackCapabilities.supports(canonical.media), isTrue);
    expect(delivery.playbackCapabilities.supportsHls, isFalse);
    expect(delivery.hlsPlaybackGateway, isNull);
    expect(gatewayInitializations, 0);
  });
}
