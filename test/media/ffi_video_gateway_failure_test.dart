import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/stub_video_gateways.dart';

void main() {
  test('maps a Rust startup exception to a safe failure', () async {
    final gateway = failingVideoGateway();

    final result = await gateway.start(AppSettings.defaults(), '/cache/native');

    expect(result, isA<VideoGatewayFailed>());
    expect(
      (result as VideoGatewayFailed).message,
      'The embedded video gateway could not start.',
    );
  });
}
