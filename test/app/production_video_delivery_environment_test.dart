import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('provides the concrete production video delivery adapters', () {
    final environment = ProductionVideoDeliveryEnvironment.production(
      MockNdk(),
      AppSettings.defaults(),
      noSignedInViewer,
    );

    expect(environment.canonicalSource, isA<NdkVideoRemoteSource>());
    expect(environment.gateway, isA<FfiVideoGateway>());
  });
}
