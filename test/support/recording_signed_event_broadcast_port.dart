import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/features/social/domain/signed_nostr_event_json.dart';

final class RecordingSignedEventBroadcastPort
    implements SignedEventBroadcastPort {
  final payloads = <SignedNostrEventJson>[];
  Object? failure;

  @override
  Future<void> broadcast(SignedNostrEventJson event) async {
    final broadcastFailure = failure;
    if (broadcastFailure != null) throw broadcastFailure;
    payloads.add(event);
  }
}
