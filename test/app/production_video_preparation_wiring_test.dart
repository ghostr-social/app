import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_remote_video_source.dart';
import '../support/feed_preparation_updates.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('progressive production delivery carries preparation updates', () async {
    final fixture = await _fixture(VideoPlaybackCapabilities.progressiveOnly);
    addTearDown(fixture.dispose);

    expect(fixture.delivery.preparationUpdates, same(fixture.updates));
  });

  test('disabled production delivery omits preparation updates', () async {
    final fixture = await _fixture(VideoPlaybackCapabilities.none);
    addTearDown(fixture.dispose);

    expect(fixture.delivery.preparationUpdates, isNull);
  });
}

Future<_PreparationFixture> _fixture(
  VideoPlaybackCapabilities capabilities,
) async {
  final root = await Directory.systemTemp.createTemp('ghostr-preparation-');
  final updates = ControlledPlaybackPreparationUpdates();
  final environment = ProductionVideoDeliveryEnvironment(
    source: FakeRemoteVideoSource([]),
    adapters: ProductionVideoDeliveryAdapters(
      supportDirectoryProvider: () async => root,
      gateway: startedVideoGateway(),
      preparationUpdates: updates,
    ),
    playbackCapabilities: capabilities,
  );
  final delivery = await buildProductionVideoDelivery(
    AppSettings.defaults(),
    environment,
  );
  return _PreparationFixture(root, updates, delivery);
}

final class _PreparationFixture {
  const _PreparationFixture(this.root, this.updates, this.delivery);

  final Directory root;
  final ControlledPlaybackPreparationUpdates updates;
  final ProductionVideoDelivery delivery;

  Future<void> dispose() async {
    await updates.close();
    await root.delete(recursive: true);
  }
}
