import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';
import 'package:ndk/ndk.dart';

class NdkNostrEventClient implements NostrEventClient {
  NdkNostrEventClient({
    required Ndk ndk,
    required List<RelayUrl> relays,
    NdkNostrEventMapper mapper = const NdkNostrEventMapper(),
  })  : _ndk = ndk,
        _relays = relays.map((relay) => relay.value).toList(),
        _mapper = mapper;

  final Ndk _ndk;
  final List<String> _relays;
  final NdkNostrEventMapper _mapper;

  @override
  NostrPublicKeyHex get publicKeyHex {
    final publicKey = _ndk.accounts.getPublicKey();
    if (publicKey == null) throw const AppFailure('Sign in first.');
    return NostrPublicKeyHex.parse(publicKey);
  }

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    try {
      final response = _ndk.requests.query(
        name: 'ghostr-event-query',
        filter: _mapper.toFilter(query),
        explicitRelays: _relays,
        timeout: const Duration(seconds: 5),
      );
      final events = await response.future;
      return events.map(_mapper.toRecord).toList();
    } on Object catch (error, stackTrace) {
      if (error is AppFailure) rethrow;
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.events',
        message: 'Could not read engagement from Nostr relays.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  @override
  Future<NostrEventId> publish(NostrUnsignedEvent event) async {
    try {
      final response = _ndk.broadcast.broadcast(
        nostrEvent: _mapper.toEvent(event, publicKeyHex),
        specificRelays: _relays,
      );
      final results = await response.broadcastDoneFuture;
      if (!results.any((result) => result.broadcastSuccessful)) {
        throw const AppFailure('No Nostr relay accepted the event.');
      }
      return NostrEventId.parse(response.publishEvent.id);
    } on Object catch (error, stackTrace) {
      if (error is AppFailure) rethrow;
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.events',
        message: 'Could not publish the Nostr event.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
