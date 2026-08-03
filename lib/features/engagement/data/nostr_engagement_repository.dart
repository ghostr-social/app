import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';
import 'package:ghostr/features/engagement/data/accepted_nostr_reaction_journal.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_reader.dart';
import 'package:ghostr/features/engagement/data/nostr_like_mutation_service.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

class NostrEngagementRepository implements NostrEngagementPort {
  factory NostrEngagementRepository(
    NostrEventClient client, {
    Duration hydrationTimeout = nostrHydrationDeadline,
  }) {
    final journal = AcceptedNostrReactionJournal();
    final reader = NostrEngagementReader(
      client,
      journal,
      hydrationTimeout: hydrationTimeout,
    );
    return NostrEngagementRepository._(
      reader,
      NostrLikeMutationService(
        client,
        reader,
        journal,
        KeyedSerialTaskQueue(),
      ),
    );
  }

  const NostrEngagementRepository._(this._reader, this._mutations);

  final NostrEngagementReader _reader;
  final NostrLikeMutationService _mutations;

  @override
  Future<VideoEngagement> load(NostrEventReference reference) {
    return _reader.load(reference);
  }

  @override
  Future<Map<NostrEventId, VideoEngagement>> loadBatch(
    List<NostrEventReference> references,
  ) {
    return _reader.loadBatch(references);
  }

  @override
  Future<VideoEngagement> setLike(
    NostrEventReference reference,
    VideoLikeIntent intent,
  ) {
    return _mutations.setLike(reference, intent);
  }
}
