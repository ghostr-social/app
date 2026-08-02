import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_file_downloader.dart';

void main() {
  test('keeps relay delivery usable when native startup reports failure',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-delivery-');
    addTearDown(() => root.delete(recursive: true));
    final delivery = await buildProductionVideoDelivery(
      AppSettings.defaults(),
      ProductionVideoDeliveryEnvironment(
        canonicalSource: FakeRemoteVideoSource([]),
        supportDirectoryProvider: () async => root,
        downloader: FakeVideoFileDownloader({}),
        gateway: _failedGateway(),
      ),
    );

    final result = delivery.remoteSource.loadRemoteFeed();

    await expectLater(result, throwsA(isA<AppFailure>()));
  });
}

FfiVideoGateway _failedGateway() {
  return FfiVideoGateway(
    initialize: () async {},
    startServer: ({
      required cacheDirectory,
      required maxParallelDownloads,
      required maxStorageBytes,
      required relayUrls,
    }) async {
      throw StateError('port unavailable');
    },
  );
}
