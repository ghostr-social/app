import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

/// The shipping transport: ndk publishes the signed event to the
/// configured relays and the write fails unless one of them accepts it.
///
/// `customSigner` is passed for parity with the pre-port call, though a
/// signed event never reaches ndk's signing branch.
class NdkBroadcastAdapter implements SignedEventBroadcastPort {
  NdkBroadcastAdapter({required Ndk ndk, required List<RelayUrl> relays})
      : _ndk = ndk,
        _relays = relays.map((relay) => relay.value).toList();

  final Ndk _ndk;
  final List<String> _relays;

  @override
  Future<void> broadcast(String signedEventJson) async {
    final response = _ndk.broadcast.broadcast(
      nostrEvent: decodeSignedNostrEvent(signedEventJson),
      specificRelays: _relays,
      customSigner: _ndk.accounts.getLoggedAccount()?.signer,
      saveToCache: false,
    );
    final results = await response.broadcastDoneFuture;
    if (!results.any((result) => result.broadcastSuccessful)) {
      throw const AppFailure('No Nostr relay accepted the event.');
    }
  }
}
