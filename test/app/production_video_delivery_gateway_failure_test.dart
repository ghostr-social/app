import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('keeps relay delivery usable when native startup reports failure',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-delivery-');
    addTearDown(() => root.delete(recursive: true));
    final relayPost = samplePost(caption: 'Relay progressive video');
    final delivery = await buildProductionVideoDelivery(
      AppSettings.defaults(),
      ProductionVideoDeliveryEnvironment(
        canonicalSource: FakeRemoteVideoSource([relayPost]),
        supportDirectoryProvider: () async => root,
        gateway: failingVideoGateway(),
      ),
    );

    final posts = await delivery.remoteSource.loadRemoteFeed();

    expect(posts, [relayPost]);
  });
}
