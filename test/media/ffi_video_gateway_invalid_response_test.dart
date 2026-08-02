import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test('maps an empty Rust endpoint to a safe failure', () async {
    final gateway = FfiVideoGateway(
      initialize: () async {},
      startServer: ({
        required cacheDirectory,
        required maxParallelDownloads,
        required maxStorageBytes,
        required relayUrls,
      }) async {
        return '   ';
      },
    );

    final result = await gateway.start(
      VideoDeliveryPlan.fromSettings(AppSettings.defaults()),
      '/cache/native',
    );

    expect(result, isA<VideoGatewayFailed>());
    expect(
      (result as VideoGatewayFailed).message,
      'The embedded video gateway returned an empty endpoint.',
    );
  });
}
