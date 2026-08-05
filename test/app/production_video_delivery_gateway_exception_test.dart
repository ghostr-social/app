import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_remote_video_source.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('fails bootstrap when Rust startup throws', () async {
    final root = await Directory.systemTemp.createTemp('ghostr-delivery-');
    addTearDown(() => root.delete(recursive: true));
    await expectLater(
      buildProductionVideoDelivery(
        AppSettings.defaults(),
        ProductionVideoDeliveryEnvironment(
          source: FakeRemoteVideoSource([]),
          adapters: ProductionVideoDeliveryAdapters(
            supportDirectoryProvider: () async => root,
            gateway: failingVideoGateway(),
          ),
        ),
      ),
      throwsA(isA<AppFailure>()),
    );
  });
}
