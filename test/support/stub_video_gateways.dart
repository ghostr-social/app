import 'package:ghostr/platform/media/ffi_video_gateway.dart';

/// A gateway whose Rust engine starts instantly on a fixed endpoint.
FfiVideoGateway startedVideoGateway({String endpoint = '127.0.0.1:3000'}) {
  return FfiVideoGateway(
    initialize: () async {},
    startEngine: (_) async => endpoint,
  );
}

/// A gateway whose Rust engine throws during startup.
FfiVideoGateway failingVideoGateway() {
  return FfiVideoGateway(
    initialize: () async {},
    startEngine: (_) async => throw StateError('port unavailable'),
  );
}
