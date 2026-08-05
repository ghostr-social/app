import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/rust_engine_configuration.dart';
import 'package:ghostr/platform/media/rust_engine_configuration_mapper.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';
import 'package:ghostr/src/rust/frb_generated.dart';

export 'rust_engine_configuration.dart';

typedef RustGatewayInitializer = Future<void> Function();
typedef RustEngineStarter = Future<String> Function(
  RustEngineStartConfiguration configuration,
);
typedef RustFfiEngineStarter = Future<String> Function({
  required String cacheDirectory,
  required FfiEngineConfiguration configuration,
});

/// Boots the Rust media engine with the FULL inventory budget — the
/// engine owns the one cache; no byte split with Dart remains.
class FfiVideoGateway {
  FfiVideoGateway({
    RustGatewayInitializer initialize = RustLib.init,
    RustEngineStarter startEngine = startRustEngine,
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
    final configuration = RustEngineStartConfiguration(
      cacheDirectory,
      RustEngineConfiguration.fromSettings(settings),
    );
    return _startEngine(configuration);
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

Future<String> startRustEngine(
  RustEngineStartConfiguration configuration, {
  RustFfiEngineStarter ffiStart = ffiStartEngine,
}) {
  return ffiStart(
    cacheDirectory: configuration.cacheDirectory,
    configuration: ffiEngineConfiguration(configuration.engine),
  );
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
