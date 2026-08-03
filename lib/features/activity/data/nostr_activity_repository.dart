import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_reaction.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';

part 'activity_source_result.dart';

class NostrActivityRepository implements ActivityRepository {
  const NostrActivityRepository({
    required NostrEventClient client,
    required AccountScopedActivityStore local,
    required FailureReporter failureReporter,
  })  : _client = client,
        _local = local,
        _failureReporter = failureReporter,
        _pinnedViewer = null;

  const NostrActivityRepository._({
    required NostrEventClient client,
    required AccountScopedActivityStore local,
    required FailureReporter failureReporter,
    required NostrPublicKeyHex viewer,
  })  : _client = client,
        _local = local,
        _failureReporter = failureReporter,
        _pinnedViewer = viewer;

  final NostrEventClient _client;
  final AccountScopedActivityStore _local;
  final FailureReporter _failureReporter;
  final NostrPublicKeyHex? _pinnedViewer;

  @override
  NostrActivityRepository snapshotForActiveAccount() {
    if (_pinnedViewer != null) return this;
    final viewer = _client.publicKeyHex;
    final local = _local.snapshotForAccount(viewer);
    return NostrActivityRepository._(
      client: _client,
      local: local,
      failureReporter: _failureReporter,
      viewer: viewer,
    );
  }

  @override
  Future<List<ActivityItem>> load() async {
    final viewer = _pinnedViewer ?? _client.publicKeyHex;
    final local = _local.snapshotForAccount(viewer);
    final sources = await Future.wait<_ActivitySourceResult>([
      _loadRemote(viewer),
      _loadLocal(local),
    ]);
    return _mergeSources(sources);
  }

  Future<_ActivitySourceResult> _loadRemote(NostrPublicKeyHex viewer) {
    return _loadActivitySource(
      _failureReporter,
      'NostrActivityRepository.loadRemote',
      () async {
        final batches = await Future.wait(
          _queries(viewer).map(_client.query),
        );
        final events = _uniqueIncoming(
          batches.expand((batch) => batch),
          viewer,
        );
        return events.map(_toItem).toList(growable: false);
      },
    );
  }

  Future<_ActivitySourceResult> _loadLocal(
    AccountScopedActivityStore local,
  ) {
    return _loadActivitySource(
      _failureReporter,
      'NostrActivityRepository.loadLocal',
      local.load,
    );
  }

  List<ActivityItem> _mergeSources(List<_ActivitySourceResult> sources) {
    final successful = sources.whereType<_ActivitySourceSuccess>().toList();
    if (successful.isEmpty) {
      throw sources.whereType<_ActivitySourceFailure>().first.failure;
    }
    final items = <String, ActivityItem>{
      for (final source in successful.reversed)
        for (final item in source.items) item.id: item,
    }.values.toList();
    items.sort((left, right) => right.occurredAt.compareTo(left.occurredAt));
    return items.take(50).toList();
  }

  @override
  Future<void> record(ActivityItem item) {
    final viewer = _pinnedViewer ?? _client.publicKeyHex;
    return _local.snapshotForAccount(viewer).record(item);
  }

  List<NostrEventQuery> _queries(NostrPublicKeyHex viewer) {
    return <NostrEventQuery>[
      _tagQuery(7, 'p', viewer),
      _tagQuery(1111, 'P', viewer),
      _tagQuery(1111, 'p', viewer),
      _tagQuery(3, 'p', viewer),
    ];
  }

  NostrEventQuery _tagQuery(
    int kind,
    String tag,
    NostrPublicKeyHex viewer,
  ) {
    return NostrEventQuery(
      kinds: <int>[kind],
      tagFilters: <NostrTagFilter>[
        NostrTagFilter(name: tag, values: <String>[viewer]),
      ],
      limit: 100,
    );
  }

  Iterable<NostrEventRecord> _uniqueIncoming(
    Iterable<NostrEventRecord> events,
    NostrPublicKeyHex viewer,
  ) {
    return <String, NostrEventRecord>{
      for (final event in events)
        if (_isIncomingActivity(event, viewer)) event.id: event,
    }.values;
  }

  bool _isIncomingActivity(
    NostrEventRecord event,
    NostrPublicKeyHex viewer,
  ) {
    return event.authorPublicKeyHex != viewer &&
        (event.kind != 7 || isNostrLikeReaction(event));
  }

  ActivityItem _toItem(NostrEventRecord event) {
    final type = _type(event.kind);
    return ActivityItem(
      id: ActivityId.parse(event.id),
      type: type,
      description: ActivityDescription(
        title: _title(type),
        body: _body(event, type),
      ),
      occurredAt: DateTime.fromMillisecondsSinceEpoch(
        event.createdAt * 1000,
        isUtc: true,
      ),
    );
  }

  ActivityType _type(int kind) {
    return switch (kind) {
      7 => ActivityType.like,
      1111 => ActivityType.comment,
      _ => ActivityType.follow,
    };
  }

  String _title(ActivityType type) {
    return switch (type) {
      ActivityType.like => 'New like',
      ActivityType.comment => 'New comment',
      _ => 'New follower',
    };
  }

  String _body(NostrEventRecord event, ActivityType type) {
    final author = _shortAuthor(event.authorPublicKeyHex);
    if (type == ActivityType.comment && event.content.trim().isNotEmpty) {
      return '$author: ${event.content.trim()}';
    }
    if (type == ActivityType.like) return '$author liked your video.';
    return '$author followed you.';
  }

  String _shortAuthor(String publicKey) {
    return publicKey.length > 12 ? '${publicKey.substring(0, 12)}…' : publicKey;
  }
}
