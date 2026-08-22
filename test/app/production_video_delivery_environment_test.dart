import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('provides the concrete production video delivery adapters', () async {
    final environment = ProductionVideoDeliveryEnvironment.production(
      noSignedInViewer,
    );

    expect(environment.source, isA<RustFeedRemoteSource>());
    expect(environment.adapters.gateway, isA<FfiVideoGateway>());
    await environment.adapters.networkStatus.close();
  });
}
