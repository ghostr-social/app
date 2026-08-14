import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_query_result_policy.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_mapper.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ghostr/src/rust/api/event_control.dart' as engine;
import 'package:ghostr/src/rust/api/event_types.dart';
import 'package:ndk/ndk.dart';

typedef RustEventQuery =
    Future<List<FfiNostrEvent>> Function({required FfiNostrEventFilter filter});

typedef RustEventBatchQuery =
    Future<List<FfiNostrEvent>> Function({
      required List<FfiNostrEventFilter> filters,
    });

const _readFailureMessage = 'Could not read from Nostr relays.';

/// Typed read side of the Rust event transport.
final class RustNostrEventQueries {
  const RustNostrEventQueries({
    this.one = engine.ffiQueryEvents,
    this.batch = engine.ffiQueryEventsBatch,
  });

  final RustEventQuery one;
  final RustEventBatchQuery batch;
}

class RustNostrEventClient
    implements NostrEventClient, SignedNostrEventPublisher {
  RustNostrEventClient({
    required Ndk ndk,
    required SignedEventBroadcastPort broadcast,
    RustNostrEventQueries queries = const RustNostrEventQueries(),
    RustNostrEventMapper mapper = const RustNostrEventMapper(),
  }) : _ndk = ndk,
       _broadcast = broadcast,
       _queries = queries,
       _mapper = mapper;

  final Ndk _ndk;
  final SignedEventBroadcastPort _broadcast;
  final RustNostrEventQueries _queries;
  final RustNostrEventMapper _mapper;

  @override
  NostrPublicKeyHex get publicKeyHex {
    final publicKey = _ndk.accounts.getPublicKey();
    if (publicKey == null) throw const AppFailure('Sign in first.');
    return NostrPublicKeyHex.parse(publicKey);
  }

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    try {
      final events = await _queries.one(filter: _mapper.toFilter(query));
      return selectNostrQueryResults(
        events: events.map(_mapper.toRecord),
        queries: <NostrEventQuery>[query],
      );
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.events',
        message: _readFailureMessage,
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
      return await _queryRequested(requested);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.events',
        message: _readFailureMessage,
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  Future<List<NostrEventRecord>> _queryRequested(
    List<NostrEventQuery> requested,
  ) async {
    final filters = requested.map(_mapper.toFilter).toList();
    final events = await _queries.batch(filters: filters);
    return selectNostrQueryResults(
      events: events.map(_mapper.toRecord),
      queries: requested,
    );
  }

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    return (await publishSigned(event, expectedAuthor: expectedAuthor)).id;
  }

  @override
  Future<NostrEventPublication> publishSigned(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    try {
      return await _publishSigned(event, expectedAuthor);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.events',
        message: 'Could not publish the Nostr event.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  Future<NostrEventPublication> _publishSigned(
    NostrUnsignedEvent event,
    NostrPublicKeyHex expectedAuthor,
  ) async {
    final signer = _requireSigner();
    final signerPublicKey = NostrPublicKeyHex.parse(signer.getPublicKey());
    _verifyExpectedAuthor(signerPublicKey, expectedAuthor);
    final unsigned = _mapper.toUnsignedEvent(event, signerPublicKey);
    final signed = await signer.sign(unsigned);
    _verifyExpectedAuthor(
      NostrPublicKeyHex.parse(signed.pubKey),
      expectedAuthor,
    );
    final json = encodeSignedNostrEvent(signed);
    await _broadcast.broadcast(json);
    return NostrEventPublication(
      id: NostrEventId.parse(signed.id),
      signedEvent: json,
    );
  }

  EventSigner _requireSigner() {
    final signer = _ndk.accounts.getLoggedAccount()?.signer;
    if (signer == null || !signer.canSign()) {
      throw const AppFailure('Sign in first.');
    }
    return signer;
  }
}

void _verifyExpectedAuthor(
  NostrPublicKeyHex actual,
  NostrPublicKeyHex expected,
) {
  if (actual != expected) {
    throw const AppFailure('The active account changed. Try again.');
  }
}
