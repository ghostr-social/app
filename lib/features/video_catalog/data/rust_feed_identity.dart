import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// Opaque handle returned by the Rust feed engine.
extension type const RustFeedId._(String value) {
  factory RustFeedId.parse(String raw) {
    final value = raw.trim();
    if (value.isEmpty) {
      throw const FormatException('Rust feed IDs cannot be empty.');
    }
    return RustFeedId._(value);
  }
}

/// Stable identity for one feed specification inside the Dart session cache.
extension type const RustFeedSpecKey._(String value) {
  factory RustFeedSpecKey.fromSpec(FfiFeedSpec spec) {
    return RustFeedSpecKey._(<String>[
      spec.kind.name,
      spec.value ?? '',
      spec.viewerPubkey ?? '',
      ...spec.creators,
    ].join('\u0000'));
  }
}

/// Monotonic token for one native Nostr account session.
extension type const RustNostrSessionGeneration._(BigInt value) {
  factory RustNostrSessionGeneration.fromBridge(BigInt value) {
    return RustNostrSessionGeneration._(value);
  }

  bool isBefore(RustNostrSessionGeneration other) => value < other.value;
}

/// Account and generation that jointly own one native feed request.
final class RustFeedAccountSession {
  const RustFeedAccountSession({
    required this.account,
    required this.generation,
  });

  final NostrPublicKeyHex? account;
  final RustNostrSessionGeneration generation;

  bool hasSameOwner(RustFeedAccountSession other) {
    return account == other.account && generation == other.generation;
  }
}
