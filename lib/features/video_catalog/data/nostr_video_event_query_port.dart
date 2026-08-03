import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ndk/ndk.dart';

abstract interface class NostrVideoEventQueryPort {
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
  });

  Future<Map<NostrPublicKeyHex, Metadata>> loadMetadataBatch(
    Set<NostrPublicKeyHex> publicKeys,
  );
}
