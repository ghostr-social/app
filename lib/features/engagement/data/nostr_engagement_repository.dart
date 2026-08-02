import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

class NostrEngagementRepository implements NostrEngagementPort {
  const NostrEngagementRepository(this._client);

  final NostrEventClient _client;

  @override
  Future<VideoEngagement> load(NostrEventReference reference) async {
    final state = await _loadState(reference);
    return state.engagementFor(_client.publicKeyHex);
  }

  @override
  Future<VideoEngagement> toggleLike(NostrEventReference reference) async {
    final state = await _loadState(reference);
    final ownReaction = state.byAuthor[_client.publicKeyHex];
    if (ownReaction == null) return _publishLike(reference, state);
    return _removeLike(ownReaction, state);
  }

  Future<_ReactionState> _loadState(NostrEventReference reference) async {
    final reactions = await _loadReactions(reference);
    final likes = reactions.where(_isLike).toList();
    final deletedIds = await _loadDeletedIds(likes);
    return _ReactionState.from(likes, deletedIds);
  }

  Future<List<NostrEventRecord>> _loadReactions(
    NostrEventReference reference,
  ) async {
    final batches = await Future.wait(
      _reactionQueries(reference).map(_client.query),
    );
    return <String, NostrEventRecord>{
      for (final event in batches.expand((events) => events)) event.id: event,
    }.values.toList();
  }

  List<NostrEventQuery> _reactionQueries(NostrEventReference reference) {
    return <NostrEventQuery>[
      NostrEventQuery(
        kinds: const <int>[7],
        scope: NostrEventQueryScope.parse(eventTags: [reference.eventId]),
      ),
      if (reference.identifier != null)
        NostrEventQuery(
          kinds: const <int>[7],
          tagFilters: [
            NostrTagFilter(name: 'a', values: [_coordinate(reference)])
          ],
        ),
    ];
  }

  Future<Set<String>> _loadDeletedIds(
    List<NostrEventRecord> reactions,
  ) async {
    if (reactions.isEmpty) return <String>{};
    final deletions = await _client.query(NostrEventQuery(
      kinds: const <int>[5],
      scope: NostrEventQueryScope.parse(
        eventTags: reactions.map((event) => event.id).toList(),
      ),
    ));
    return _validDeletedIds(reactions, deletions);
  }

  Set<String> _validDeletedIds(
    List<NostrEventRecord> reactions,
    List<NostrEventRecord> deletions,
  ) {
    final authorsById = <String, String>{
      for (final reaction in reactions)
        reaction.id: reaction.authorPublicKeyHex,
    };
    return deletions.expand((deletion) {
      return deletion.tagValues('e').where((id) {
        return authorsById[id] == deletion.authorPublicKeyHex;
      });
    }).toSet();
  }

  Future<VideoEngagement> _publishLike(
    NostrEventReference reference,
    _ReactionState state,
  ) async {
    await _client.publish(NostrUnsignedEvent(
      kind: 7,
      tags: _reactionTags(reference),
      content: '+',
    ));
    return VideoEngagement(
      likeCount: state.byAuthor.length + 1,
      viewerHasLiked: true,
    );
  }

  List<List<String>> _reactionTags(NostrEventReference reference) {
    return <List<String>>[
      <String>['e', reference.eventId],
      <String>['p', reference.authorPublicKeyHex],
      <String>['k', '${reference.kind}'],
      if (reference.identifier != null) <String>['a', _coordinate(reference)],
    ];
  }

  String _coordinate(NostrEventReference reference) {
    return '${reference.kind}:${reference.authorPublicKeyHex}:${reference.identifier}';
  }

  Future<VideoEngagement> _removeLike(
    NostrEventRecord reaction,
    _ReactionState state,
  ) async {
    await _client.publish(NostrUnsignedEvent(
      kind: 5,
      tags: <List<String>>[
        <String>['e', reaction.id],
        const <String>['k', '7'],
      ],
      content: 'Removed like',
    ));
    return VideoEngagement(
      likeCount: state.byAuthor.length - 1,
      viewerHasLiked: false,
    );
  }

  bool _isLike(NostrEventRecord event) {
    return event.content.isEmpty || event.content == '+';
  }
}

class _ReactionState {
  const _ReactionState(this.byAuthor);

  factory _ReactionState.from(
    List<NostrEventRecord> reactions,
    Set<String> deletedIds,
  ) {
    final active = reactions.where((event) => !deletedIds.contains(event.id));
    final sorted = active.toList()
      ..sort((left, right) => left.createdAt.compareTo(right.createdAt));
    return _ReactionState(<String, NostrEventRecord>{
      for (final reaction in sorted) reaction.authorPublicKeyHex: reaction,
    });
  }

  final Map<String, NostrEventRecord> byAuthor;

  VideoEngagement engagementFor(String viewerPublicKey) {
    return VideoEngagement(
      likeCount: byAuthor.length,
      viewerHasLiked: byAuthor.containsKey(viewerPublicKey),
    );
  }
}
