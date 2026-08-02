import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';
import 'package:ghostr/src/rust/frb_generated.dart';
import 'package:ghostr/src/rust/video/video.dart';

typedef RustGatewayInitializer = Future<void> Function();
typedef RustGatewayStarter = Future<String> Function({
  required String cacheDirectory,
  required BigInt maxParallelDownloads,
  required BigInt maxStorageBytes,
  required String relayUrls,
});

class FfiVideoGateway {
  FfiVideoGateway({
    RustGatewayInitializer initialize = RustLib.init,
    RustGatewayStarter startServer = ffiStartServer,
  })  : _initialize = initialize,
        _startServer = startServer;

  final RustGatewayInitializer _initialize;
  final RustGatewayStarter _startServer;

  Future<VideoGatewayStartResult> start(
    VideoDeliveryPlan plan,
    String cacheDirectory,
  ) async {
    try {
      await _initialize();
      final endpoint = await _start(plan, cacheDirectory);
      return _endpointResult(endpoint);
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'ghostr.video.gateway',
        message: 'The embedded video gateway could not start.',
        error: error,
        stackTrace: stackTrace,
      );
      return VideoGatewayFailed(failure.message);
    }
  }

  Future<String> _start(VideoDeliveryPlan plan, String cacheDirectory) {
    return _startServer(
      cacheDirectory: cacheDirectory,
      maxParallelDownloads: BigInt.from(8),
      maxStorageBytes: BigInt.from(plan.nativeCacheBytes),
      relayUrls: plan.relayUrls.map((relay) => relay.value).join('\n'),
    );
  }

  VideoGatewayStartResult _endpointResult(String rawEndpoint) {
    final endpoint = rawEndpoint.trim();
    if (endpoint.isNotEmpty) {
      return VideoGatewayStarted(endpoint);
    }
    return const VideoGatewayFailed(
      'The embedded video gateway returned an empty endpoint.',
    );
  }
}

sealed class VideoGatewayStartResult {
  const VideoGatewayStartResult();
}

class VideoGatewayStarted extends VideoGatewayStartResult {
  const VideoGatewayStarted(this.endpoint);

  final String endpoint;
}

class VideoGatewayFailed extends VideoGatewayStartResult {
  const VideoGatewayFailed(this.message);

  final String message;
}
