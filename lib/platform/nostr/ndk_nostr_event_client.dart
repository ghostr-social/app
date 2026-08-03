import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_query_result_policy.dart';
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
      return _acceptedRecords(events, <NostrEventQuery>[query]);
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
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> queries,
  ) async {
    final requested = List<NostrEventQuery>.unmodifiable(queries);
    if (requested.isEmpty) return const <NostrEventRecord>[];
    if (requested.length > 20) {
      throw const AppFailure('Nostr filter batch exceeds the safe limit.');
    }
    try {
      final response = _ndk.requests.query(
        name: 'ghostr-event-batch-query',
        // NDK 0.8.3 has no non-deprecated multi-filter REQ alternative.
        // ignore: deprecated_member_use
        filters: requested.map(_mapper.toFilter).toList(),
        explicitRelays: _relays,
        timeout: const Duration(seconds: 5),
      );
      final events = await response.future;
      return _acceptedRecords(events, requested);
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
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    try {
      final signer = _requireSigner();
      final signerPublicKey = NostrPublicKeyHex.parse(signer.getPublicKey());
      if (signerPublicKey != expectedAuthor) {
        throw const AppFailure('The active account changed. Try again.');
      }
      final unsigned = _mapper.toEvent(event, signerPublicKey);
      final signed = await signer.sign(unsigned);
      final response = _ndk.broadcast.broadcast(
        nostrEvent: signed,
        specificRelays: _relays,
        saveToCache: false,
      );
      final results = await response.broadcastDoneFuture;
      if (!results.any((result) => result.broadcastSuccessful)) {
        throw const AppFailure('No Nostr relay accepted the event.');
      }
      await _cacheAccepted(response.publishEvent);
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

  EventSigner _requireSigner() {
    final signer = _ndk.accounts.getLoggedAccount()?.signer;
    if (signer == null || !signer.canSign()) {
      throw const AppFailure('Sign in first.');
    }
    return signer;
  }

  List<NostrEventRecord> _acceptedRecords(
    Iterable<Nip01Event> events,
    List<NostrEventQuery> queries,
  ) {
    // A malformed relay event aborts the boundary call instead of entering a
    // partially trusted result set beside validated domain records.
    return selectNostrQueryResults(
      events: events.map(_mapper.toRecord),
      queries: queries,
    );
  }

  Future<void> _cacheAccepted(Nip01Event event) async {
    try {
      await _ndk.config.cache.saveEvent(event);
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.nostr.events.cache',
        message: 'An accepted Nostr event could not be cached locally.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
