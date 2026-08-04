/// Publishes events that are already signed.
///
/// Signing stays in Dart and keys never leave it (plan §5 step 5): a
/// mutation builds its event, signs it, and hands the canonical NIP-01
/// JSON to this port. The transport behind it — ndk today, the Rust
/// engine once discovery moves — only relays bytes.
abstract interface class SignedEventBroadcastPort {
  /// Publishes one signed event, or throws when no relay accepted it.
  Future<void> broadcast(String signedEventJson);
}
