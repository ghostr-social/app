import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery_infrastructure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test('retired cache deletion failure does not block the Rust gateway',
      () async {
    final root = await Directory.systemTemp.createTemp('ghostr-retired-cache-');
    addTearDown(() => root.delete(recursive: true));
    final gateway = FfiVideoGateway(
      initialize: () async {},
      startEngine: (_) async => '127.0.0.1:3000',
    );

    final result = await initializeProductionVideoDeliveryInfrastructure(
      settings: AppSettings.defaults(),
      directoryProvider: () async => root,
      gateway: gateway,
      removeRetiredCache: (_) async => throw const FileSystemException(),
    );

    expect(result, isA<VideoGatewayStarted>());
  });
}
