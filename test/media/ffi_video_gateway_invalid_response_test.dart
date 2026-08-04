import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/stub_video_gateways.dart';

void main() {
  test('maps an empty Rust endpoint to a safe failure', () async {
    final gateway = startedVideoGateway(endpoint: '   ');

    final result = await gateway.start(AppSettings.defaults(), '/cache/native');

    expect(result, isA<VideoGatewayFailed>());
    expect(
      (result as VideoGatewayFailed).message,
      'The embedded video gateway returned an empty endpoint.',
    );
  });
}
