import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';

/// Every NIP-71 video kind: normal + short, current + deprecated addressable.
const List<int> videoEventKinds = [21, 22, 34235, 34236];

/// Mime values worth asking NIP-94 file events for, via the `#m` filter.
const List<String> videoFileMimeTypes = [
  'video/mp4',
  'video/webm',
  'video/quicktime',
  'video/mpeg',
  'application/x-mpegurl',
  'application/vnd.apple.mpegurl',
];

typedef _DiscoveryRequest = ({
  Set<NostrPublicKeyHex>? authors,
  String? searchQuery,
  Set<String>? hashtags,
  DateTime? olderThan,
  bool wide,
});

typedef _QuerySpec = ({
  List<int> kinds,
  int limit,
  String? search,
  List<NostrTagFilter> tagFilters,
});

/// Builds the relay queries answering one video discovery request.
///
/// Every request pairs the dedicated video kinds with a kind-1 note window
/// (most Nostr videos travel as plain notes with a link), a NIP-94 kind-1063
/// query filtered server-side to video mimes, and — when the viewer gave no
/// term of their own — a NIP-50 hunt for notes that literally mention a
/// video file, so the search relays pre-filter instead of blind luck.
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
  final tags = _hashtagFilters(hashtags);
  return List<NostrEventQuery>.unmodifiable([
    _build(request, (
      kinds: videoEventKinds,
      limit: request.wide ? 200 : 80,
      search: searchQuery,
      tagFilters: tags,
    )),
    _build(request, (
      kinds: const [1],
      limit: 200,
      search: searchQuery,
      tagFilters: tags,
    )),
    if (searchQuery == null)
      _build(request, (
        kinds: const [1],
        limit: 200,
        search: 'mp4',
        tagFilters: tags,
      )),
    _build(request, (
      kinds: const [1063],
      limit: 200,
      search: searchQuery,
      tagFilters: [
        NostrTagFilter(name: 'm', values: videoFileMimeTypes),
        ...tags,
      ],
    )),
  ]);
}

List<NostrTagFilter> _hashtagFilters(Set<String>? hashtags) {
  if (hashtags == null || hashtags.isEmpty) return const <NostrTagFilter>[];
  return [
    NostrTagFilter(
      name: 't',
      values: hashtags.expand(hashtagQueryVariants).toSet().toList(),
    ),
  ];
}

NostrEventQuery _build(_DiscoveryRequest request, _QuerySpec spec) {
  return NostrEventQuery(
    kinds: spec.kinds,
    scope: NostrEventQueryScope(
      authors: request.authors?.toList() ?? const <NostrPublicKeyHex>[],
    ),
    tagFilters: spec.tagFilters,
    limit: spec.limit,
    until: request.olderThan == null
        ? null
        : request.olderThan!.toUtc().millisecondsSinceEpoch ~/ 1000,
    search: spec.search,
  );
}
