import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';

/// Every NIP-71 video kind: normal + short, current + deprecated addressable.
const List<int> videoEventKinds = [21, 22, 34235, 34236];

typedef _DiscoveryRequest = ({
  Set<NostrPublicKeyHex>? authors,
  String? searchQuery,
  Set<String>? hashtags,
  DateTime? olderThan,
  bool wide,
});

/// Builds the relay queries answering one video discovery request.
///
/// Every request pairs the dedicated video kinds with a kind-1 note query,
/// because most video content on Nostr is published as ordinary notes
/// carrying a video link. The note window is always wide: only a fraction
/// of notes turn out to carry a playable video.
List<NostrEventQuery> videoDiscoveryQueries({
  Set<NostrPublicKeyHex>? authorPublicKeys,
  String? searchQuery,
  Set<String>? hashtags,
  DateTime? olderThan,
}) {
  final request = (
    authors: authorPublicKeys,
    searchQuery: searchQuery,
    hashtags: hashtags,
    olderThan: olderThan,
    wide: searchQuery != null || (hashtags?.isNotEmpty ?? false),
  );
  return List<NostrEventQuery>.unmodifiable([
    _query(videoEventKinds, request, limit: request.wide ? 200 : 80),
    _query(const [1], request, limit: 200),
  ]);
}

NostrEventQuery _query(
  List<int> kinds,
  _DiscoveryRequest request, {
  required int limit,
}) {
  return NostrEventQuery(
    kinds: kinds,
    scope: NostrEventQueryScope(
      authors: request.authors?.toList() ?? const <NostrPublicKeyHex>[],
    ),
    tagFilters: [
      if (request.hashtags case final tags? when tags.isNotEmpty)
        NostrTagFilter(
          name: 't',
          values: tags.expand(hashtagQueryVariants).toSet().toList(),
        ),
    ],
    limit: limit,
    until: request.olderThan == null
        ? null
        : request.olderThan!.toUtc().millisecondsSinceEpoch ~/ 1000,
    search: request.searchQuery,
  );
}
