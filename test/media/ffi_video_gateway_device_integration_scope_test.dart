import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test('loopback device integration is explicit per engine start', () async {
    final starts = <RustEngineStartConfiguration>[];
    final gateway = FfiVideoGateway(
      initialize: () async {},
      startEngine: (configuration) async {
        starts.add(configuration);
        return '127.0.0.1:3000';
      },
    );

    await gateway.start(AppSettings.defaults(), '/cache/ordinary');
    await gateway.start(
      AppSettings.defaults(),
      '/cache/journey',
      deviceIntegrationOrigin: Uri.parse('http://127.0.0.1:4040/video.mp4'),
    );

    expect(starts.first.deviceIntegrationOrigin, isNull);
    expect(starts.last.deviceIntegrationOrigin, 'http://127.0.0.1:4040');
  });
}
