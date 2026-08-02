import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';

class NostrActivityRepository implements ActivityRepository {
  const NostrActivityRepository({
    required NostrEventClient client,
    required ActivityRepository local,
  })  : _client = client,
        _local = local;

  final NostrEventClient _client;
  final ActivityRepository _local;

  @override
  Future<List<ActivityItem>> load() async {
    final batches = await Future.wait(
      _queries(_client.publicKeyHex).map(_client.query),
    );
    final events = _uniqueIncoming(batches.expand((batch) => batch));
    final localItems = await _local.load();
    final items = <String, ActivityItem>{
      for (final item in localItems) item.id: item,
      for (final event in events) event.id: _toItem(event),
    }.values.toList();
    items.sort((left, right) => right.occurredAt.compareTo(left.occurredAt));
    return items.take(50).toList();
  }

  @override
  Future<void> record(ActivityItem item) => _local.record(item);

  List<NostrEventQuery> _queries(String viewer) {
    return <NostrEventQuery>[
      _tagQuery(7, 'p', viewer),
      _tagQuery(1111, 'P', viewer),
      _tagQuery(1111, 'p', viewer),
      _tagQuery(3, 'p', viewer),
    ];
  }

  NostrEventQuery _tagQuery(int kind, String tag, String viewer) {
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
  ) {
    return <String, NostrEventRecord>{
      for (final event in events)
        if (event.authorPublicKeyHex != _client.publicKeyHex) event.id: event,
    }.values;
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
