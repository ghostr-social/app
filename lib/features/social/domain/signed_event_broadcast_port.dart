import 'package:ghostr/features/social/domain/signed_nostr_event_json.dart';

/// Publishes events that are already signed.
///
/// Signing stays in Dart and keys never leave it (plan §5 step 5): a
/// mutation builds its event, signs it, and hands the canonical NIP-01
/// JSON to this port. The Rust engine only relays those signed bytes.
abstract interface class SignedEventBroadcastPort {
  /// Publishes one signed event, or throws when no relay accepted it.
  Future<void> broadcast(SignedNostrEventJson event);
}
