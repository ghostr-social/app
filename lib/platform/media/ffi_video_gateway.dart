import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';
import 'package:ghostr/src/rust/frb_generated.dart';

typedef RustGatewayInitializer = Future<void> Function();
typedef RustEngineStarter = Future<String> Function({
  required String cacheDirectory,
  required String relayUrls,
  required String dataUsage,
  required BigInt maxStorageBytes,
});

/// Boots the Rust media engine with the FULL inventory budget — the
/// engine owns the one cache; no byte split with Dart remains.
class FfiVideoGateway {
  FfiVideoGateway({
    RustGatewayInitializer initialize = RustLib.init,
    RustEngineStarter startEngine = ffiStartEngine,
  })  : _initialize = initialize,
        _startEngine = startEngine;

  final RustGatewayInitializer _initialize;
  final RustEngineStarter _startEngine;

  Future<VideoGatewayStartResult> start(
    AppSettings settings,
    String cacheDirectory,
  ) async {
    try {
      await _initialize();
      final endpoint = await _start(settings, cacheDirectory);
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

  Future<String> _start(AppSettings settings, String cacheDirectory) {
    return _startEngine(
      cacheDirectory: cacheDirectory,
      relayUrls: settings.relays.map((relay) => relay.value).join('\n'),
      dataUsage: settings.dataUsage.name,
      maxStorageBytes: BigInt.from(settings.inventoryBudget.bytes),
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
