import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_file_downloader.dart';

void main() {
  test('enforces both cache partitions before delivery becomes available',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-delivery-');
    addTearDown(() => root.delete(recursive: true));
    final dartCache = Directory('${root.path}/video_inventory');
    final nativeCache = Directory('${root.path}/native_video_inventory');
    await dartCache.create(recursive: true);
    await nativeCache.create(recursive: true);
    final oversized = File('${dartCache.path}/oversized.video');
    oversized.openSync(mode: FileMode.write).truncateSync(129 * 1024 * 1024);
    final staleNative = File('${nativeCache.path}/stale.mp4');
    await staleNative.writeAsBytes([1]);
    final environment = ProductionVideoDeliveryEnvironment(
      canonicalSource: FakeRemoteVideoSource([]),
      supportDirectoryProvider: () async => root,
      downloader: FakeVideoFileDownloader({}),
      gateway: _startedGateway(),
    );

    await buildProductionVideoDelivery(
      AppSettings.defaults().copyWith(
        inventoryBudget: VideoInventoryBudget.twoHundredFiftySixMegabytes,
      ),
      environment,
    );

    expect(await oversized.exists(), isFalse);
    expect(await nativeCache.list().toList(), isEmpty);
  });
}

FfiVideoGateway _startedGateway() {
  return FfiVideoGateway(
    initialize: () async {},
    startServer: ({
      required cacheDirectory,
      required maxParallelDownloads,
      required maxStorageBytes,
      required relayUrls,
    }) async {
      return '127.0.0.1:3000';
    },
  );
}
