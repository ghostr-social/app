import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('provides the concrete production video delivery adapters', () {
    final environment = ProductionVideoDeliveryEnvironment.production(
      MockNdk(),
    );

    expect(environment.canonicalSource, isA<NdkVideoRemoteSource>());
    expect(environment.downloader, isA<HttpVideoFileDownloader>());
    expect(environment.gateway, isA<FfiVideoGateway>());
  });
}
