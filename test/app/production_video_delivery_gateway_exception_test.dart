import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_remote_video_source.dart';
import '../support/stub_video_gateways.dart';

void main() {
  // The native fallback shim is retired (plan §4 step 10), so a gateway
  // startup exception must not turn an empty relay feed into an error.
  test('serves an empty relay feed as empty when gateway startup throws',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-delivery-');
    addTearDown(() => root.delete(recursive: true));
    final delivery = await buildProductionVideoDelivery(
      AppSettings.defaults(),
      ProductionVideoDeliveryEnvironment(
        canonicalSource: FakeRemoteVideoSource([]),
        supportDirectoryProvider: () async => root,
        gateway: failingVideoGateway(),
      ),
    );

    final posts = await delivery.remoteSource.loadRemoteFeed();

    expect(posts, isEmpty);
  });
}
