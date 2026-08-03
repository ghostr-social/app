import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

class _RevisionQuery implements NostrVideoEventQueryPort {
  const _RevisionQuery(this.events);

  final List<Nip01Event> events;

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async =>
      events;

  @override
  Future<Map<NostrPublicKeyHex, Metadata>> loadMetadataBatch(
    Set<NostrPublicKeyHex> publicKeys,
  ) async =>
      const {};
}

void main() {
  test('keeps only the newest revision of an addressable NIP-71 video',
      () async {
    final source = NdkVideoRemoteSource(_RevisionQuery([
      _revision(testEventId, 10, 'stale revision'),
      _revision(secondTestEventId, 20, 'current revision'),
      _revision(publishedEventId(1), 20, 'canonical tie winner'),
      _blankIdentifierRevision(),
    ]));

    final posts = await source.loadRemoteFeed();

    expect(posts, hasLength(1));
    expect(posts.single.id.value, publishedEventId(1));
    expect(posts.single.caption, 'canonical tie winner');
  });
}

Nip01Event _blankIdentifierRevision() {
  return Nip01Event(
    id: publishedTestEventId,
    pubKey: testCreatorPublicKey,
    kind: 34236,
    createdAt: 30,
    content: 'malformed revision',
    tags: const [
      ['d', '  '],
      ['imeta', 'url https://media.example/video.mp4', 'm video/mp4'],
    ],
  );
}

Nip01Event _revision(String id, int createdAt, String content) {
  return Nip01Event(
    id: id,
    pubKey: testCreatorPublicKey,
    kind: 34236,
    createdAt: createdAt,
    content: content,
    tags: const [
      ['d', 'same-video'],
      ['imeta', 'url https://media.example/video.mp4', 'm video/mp4'],
    ],
  );
}
