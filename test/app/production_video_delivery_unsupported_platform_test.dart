import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_file_downloader.dart';
import '../support/sample_data.dart';

void main() {
  test('disables media startup when the platform has no player backend',
      () async {
    var directoryRequests = 0;
    var gatewayInitializations = 0;
    var gatewayStarts = 0;
    final canonical = FakeRemoteVideoSource([samplePost()]);
    final delivery = await buildProductionVideoDelivery(
      AppSettings.defaults(),
      ProductionVideoDeliveryEnvironment(
        canonicalSource: canonical,
        supportDirectoryProvider: () {
          directoryRequests += 1;
          throw StateError('filesystem must stay idle');
        },
        downloader: FakeVideoFileDownloader({}),
        gateway: FfiVideoGateway(
          initialize: () async {
            gatewayInitializations += 1;
          },
          startEngine: ({
            required String cacheDirectory,
            required String relayUrls,
            required String dataUsage,
            required BigInt maxStorageBytes,
          }) async {
            gatewayStarts += 1;
            return '127.0.0.1:3000';
          },
        ),
        playbackCapabilities: VideoPlaybackCapabilities.none,
      ),
    );

    expect(directoryRequests, 0);
    expect(gatewayInitializations, 0);
    expect(gatewayStarts, 0);
    expect(
      delivery.remoteSource.loadRemoteFeed,
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Video playback is unavailable on this platform.',
        ),
      ),
    );
    expect(canonical.loadCount, 0);
  });
}
