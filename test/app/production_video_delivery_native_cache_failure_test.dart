import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/fake_remote_video_source.dart';

void main() {
  test('fails bootstrap when the Rust cache cannot be prepared', () async {
    final root = await Directory.systemTemp.createTemp('ghostr-native-fail-');
    addTearDown(() => root.delete(recursive: true));
    await File('${root.path}/native_video_inventory').writeAsString('occupied');
    var gatewayInitializations = 0;
    await expectLater(
      buildProductionVideoDelivery(
        AppSettings.defaults(),
        ProductionVideoDeliveryEnvironment(
          source: FakeRemoteVideoSource([]),
          adapters: ProductionVideoDeliveryAdapters(
            supportDirectoryProvider: () async => root,
            gateway: FfiVideoGateway(
              initialize: () async {
                gatewayInitializations += 1;
              },
              startEngine: (_) async => '127.0.0.1:3000',
            ),
          ),
          playbackCapabilities: VideoPlaybackCapabilities.progressiveAndHls,
        ),
      ),
      throwsA(isA<AppFailure>()),
    );

    expect(gatewayInitializations, 0);
  });
}
