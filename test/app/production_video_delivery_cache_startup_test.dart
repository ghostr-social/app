import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_remote_video_source.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('removes the retired Dart cache and keeps the native store', () async {
    final root = await Directory.systemTemp.createTemp('ghostr-delivery-');
    addTearDown(() => root.delete(recursive: true));
    final dartCache = Directory('${root.path}/video_inventory');
    final nativeCache = Directory('${root.path}/native_video_inventory');
    await dartCache.create(recursive: true);
    await Directory('${nativeCache.path}/progressive').create(recursive: true);
    // The Dart store is gone, so nothing drains this partition any more:
    // the whole directory must leave with it.
    await File('${dartCache.path}/stale.video').writeAsBytes([1, 2, 3]);
    // The engine reloads these on start, so startup must not disturb them.
    final cached = File('${nativeCache.path}/progressive/post.part');
    await cached.writeAsBytes([1]);
    final environment = ProductionVideoDeliveryEnvironment(
      canonicalSource: FakeRemoteVideoSource([]),
      supportDirectoryProvider: () async => root,
      gateway: startedVideoGateway(),
    );

    await buildProductionVideoDelivery(AppSettings.defaults(), environment);

    expect(await dartCache.exists(), isFalse);
    expect(await cached.exists(), isTrue);
  });
}
