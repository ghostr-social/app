import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/src/rust/api/broadcast_control.dart' as engine;

typedef RustEventBroadcast = Future<void> Function({
  required String signedEventJson,
});

/// The Rust transport: the engine re-validates the event and fans it out
/// to the author's outbox relays. Only signed bytes cross the FFI.
class RustBroadcastAdapter implements SignedEventBroadcastPort {
  const RustBroadcastAdapter({
    RustEventBroadcast send = engine.ffiBroadcastEvent,
  }) : _send = send;

  final RustEventBroadcast _send;

  @override
  Future<void> broadcast(String signedEventJson) async {
    try {
      await _send(signedEventJson: signedEventJson);
    } on Object catch (error, stackTrace) {
      // Callers only know relay rejection, so an engine or validation
      // error has to read as the same refused write.
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.broadcast.rust',
        message: 'No Nostr relay accepted the event.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
