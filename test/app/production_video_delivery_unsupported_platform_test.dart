import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('starts Rust but disables playback when no player backend exists',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-engine-only-');
    addTearDown(() => root.delete(recursive: true));
    var directoryRequests = 0;
    var gatewayInitializations = 0;
    var gatewayStarts = 0;
    final canonical = FakeRemoteVideoSource([samplePost()]);
    final delivery = await buildProductionVideoDelivery(
      AppSettings.defaults(),
      ProductionVideoDeliveryEnvironment(
        source: canonical,
        adapters: ProductionVideoDeliveryAdapters(
          supportDirectoryProvider: () async {
            directoryRequests += 1;
            return root;
          },
          gateway: FfiVideoGateway(
            initialize: () async {
              gatewayInitializations += 1;
            },
            startEngine: (_) async {
              gatewayStarts += 1;
              return '127.0.0.1:3000';
            },
          ),
        ),
        playbackCapabilities: VideoPlaybackCapabilities.none,
      ),
    );

    expect(directoryRequests, 1);
    expect(gatewayInitializations, 1);
    expect(gatewayStarts, 1);
    expect(
      delivery.remoteSource.loadRemoteFeed,
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Video playback is unavailable on this platform.',
        ),
      ),
    );
    expect(canonical.loadCount, 0);
  });
}
