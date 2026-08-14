import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';
import 'package:ghostr/features/reposts/data/accepted_nostr_repost_journal.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_mutation_service.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_reader.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

final class NostrVideoRepostRepository implements VideoRepostRepository {
  factory NostrVideoRepostRepository(
    NostrEventClient client, {
    required NostrRepostRelayHint relayHint,
    Duration timeout = const Duration(seconds: 10),
    Duration hydrationTimeout = const Duration(milliseconds: 100),
  }) {
    final journal = AcceptedNostrRepostJournal();
    final hydrationReader = NostrRepostReader(
      client,
      journal,
      timeout: hydrationTimeout,
    );
    final mutationReader = NostrRepostReader(client, journal, timeout: timeout);
    final mutations = NostrRepostMutationService(
      NostrRepostMutationDependencies(
        client,
        mutationReader,
        journal,
        KeyedSerialTaskQueue(),
      ),
      relayHint: relayHint,
    );
    return NostrVideoRepostRepository._(
      hydrationReader,
      mutationReader,
      mutations,
      timeout,
    );
  }

  const NostrVideoRepostRepository._(
    this._reader,
    this._patientReader,
    this._mutations,
    this._patientTimeout,
  );

  final NostrRepostReader _reader;
  final NostrRepostReader _patientReader;
  final NostrRepostMutationService _mutations;
  final Duration _patientTimeout;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) {
    return mode == VideoRepostHydration.patient
        ? _hydratePatient(posts)
        : _hydratePrompt(posts);
  }

  Future<List<VideoPost>> _hydratePrompt(List<VideoPost> posts) async {
    try {
      return await _hydrateChunk(posts, _reader);
    } on AppFailure {
      return posts;
    }
  }

  Future<List<VideoPost>> _hydratePatient(List<VideoPost> posts) async {
    final hydrated = <VideoPost>[];
    final budget = NostrQueryBudget(_patientTimeout);
    final viewer = _patientReader.viewer;
    for (
      var offset = 0;
      offset < posts.length;
      offset += maxNostrTargetsPerFamily
    ) {
      final chunk = posts.skip(offset).take(maxNostrTargetsPerFamily).toList();
      try {
        _patientReader.verifyViewer(viewer);
        hydrated.addAll(await _hydrateChunk(chunk, _patientReader, budget));
        _patientReader.verifyViewer(viewer);
      } on AppFailure {
        _patientReader.verifyViewer(viewer);
        hydrated.addAll(posts.skip(offset));
        break;
      }
    }
    return List<VideoPost>.unmodifiable(hydrated);
  }

  Future<List<VideoPost>> _hydrateChunk(
    List<VideoPost> posts,
    NostrRepostReader reader, [
    NostrQueryBudget? budget,
  ]) async {
    final references = posts
        .map((post) => post.nostrReference)
        .whereType<NostrEventReference>()
        .toList(growable: false);
    if (references.isEmpty) return posts;
    final states = await reader.loadBatch(references, budget: budget);
    return posts.map((post) => _hydrated(post, states)).toList(growable: false);
  }

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async {
    final reference = post.nostrReference;
    if (reference == null) {
      throw const AppFailure('This video has no Nostr event to repost.');
    }
    final intent = post.viewerHasReposted
        ? VideoRepostIntent.remove
        : VideoRepostIntent.repost;
    final state = await _mutations.setRepost(reference, intent);
    return post.withRepost(
      state.viewerHasReposted,
      observation: VideoRepostObservation.observed,
    );
  }

  VideoPost _hydrated(
    VideoPost post,
    Map<NostrEventId, NostrViewerRepostState> states,
  ) {
    final reference = post.nostrReference;
    if (reference == null) return post;
    final state = states[reference.eventId];
    if (state == null) return post;
    return post.withRepost(
      state.viewerHasReposted,
      observation: VideoRepostObservation.observed,
    );
  }
}
