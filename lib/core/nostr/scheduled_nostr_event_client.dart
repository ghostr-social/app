import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';

/// Queues relay reads behind the shared retrieval pool as enrichment work.
///
/// Publishes bypass the queue: a like or comment the viewer just made must
/// reach relays immediately, never behind background fetching.
final class ScheduledNostrEventClient implements NostrEventClient {
  const ScheduledNostrEventClient({
    required NostrEventClient client,
    required RetrievalScheduler scheduler,
    String context = 'engagement',
  })  : _client = client,
        _scheduler = scheduler,
        _context = context;

  final NostrEventClient _client;
  final RetrievalScheduler _scheduler;
  final String _context;

  RetrievalRequest get _request => RetrievalRequest(
        context: _context,
        priority: RetrievalPriority.enrichment,
      );

  @override
  NostrPublicKeyHex get publicKeyHex => _client.publicKeyHex;

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) {
    return _scheduler.run(_request, () => _client.query(query));
  }

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> queries) {
    return _scheduler.run(_request, () => _client.queryBatch(queries));
  }

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) {
    return _client.publish(event, expectedAuthor: expectedAuthor);
  }
}
