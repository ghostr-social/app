import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_file_downloader.dart';
import '../support/stub_video_gateways.dart';

void main() {
  test('drains the legacy Dart cache and clears the native partition',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-delivery-');
    addTearDown(() => root.delete(recursive: true));
    final dartCache = Directory('${root.path}/video_inventory');
    final nativeCache = Directory('${root.path}/native_video_inventory');
    await dartCache.create(recursive: true);
    await nativeCache.create(recursive: true);
    // The full budget belongs to the Rust engine now, so even a small
    // stale file must leave the retired Dart partition.
    final stale = File('${dartCache.path}/stale.video');
    await stale.writeAsBytes([1, 2, 3]);
    final staleNative = File('${nativeCache.path}/stale.mp4');
    await staleNative.writeAsBytes([1]);
    final environment = ProductionVideoDeliveryEnvironment(
      canonicalSource: FakeRemoteVideoSource([]),
      supportDirectoryProvider: () async => root,
      downloader: FakeVideoFileDownloader({}),
      gateway: startedVideoGateway(),
    );

    await buildProductionVideoDelivery(AppSettings.defaults(), environment);

    expect(await stale.exists(), isFalse);
    expect(await nativeCache.list().toList(), isEmpty);
  });
}
