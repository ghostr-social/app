import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test('maps a Rust startup exception to a safe failure', () async {
    final gateway = FfiVideoGateway(
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

    final result = await gateway.start(
      VideoDeliveryPlan.fromSettings(AppSettings.defaults()),
      '/cache/native',
    );

    expect(result, isA<VideoGatewayFailed>());
    expect(
      (result as VideoGatewayFailed).message,
      'The embedded video gateway could not start.',
    );
  });
}
