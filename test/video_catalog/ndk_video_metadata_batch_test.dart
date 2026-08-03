import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ndk/ndk.dart';

class _BatchQuery implements NostrVideoEventQueryPort {
  _BatchQuery(this.events);

  final List<Nip01Event> events;
  int metadataCalls = 0;
  Set<NostrPublicKeyHex> requestedMetadata = {};

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
    Set<String>? hashtags,
  }) async =>
      events;

  @override
  Future<Map<NostrPublicKeyHex, Metadata>> loadMetadataBatch(
    Set<NostrPublicKeyHex> publicKeys,
  ) async {
    metadataCalls += 1;
    requestedMetadata = publicKeys;
    final missing = publicKeys.last;
    return {
      for (final key in publicKeys)
        if (key != missing)
          key: Metadata(pubKey: key.value, displayName: 'Creator'),
    };
  }
}

void main() {
  test('loads metadata for an 80-author feed through one batch', () async {
    final query = _BatchQuery([
      ...List.generate(80, _event),
      _malformedAuthorEvent(),
    ]);

    final posts = await NdkVideoRemoteSource(query).loadRemoteFeed();

    expect(posts, hasLength(80));
    expect(query.metadataCalls, 1);
    expect(query.requestedMetadata, hasLength(80));
    expect(
      posts
          .singleWhere((post) => post.caption == 'Video 79')
          .creator
          .displayName,
      startsWith('npub1'),
    );
  });
}

Nip01Event _malformedAuthorEvent() {
  return Nip01Event(
    id: 'f'.padLeft(64, 'f'),
    pubKey: 'malformed',
    kind: 22,
    content: 'Invalid creator',
    tags: const [
      ['imeta', 'url https://cdn.example/invalid.mp4', 'm video/mp4'],
    ],
  );
}

Nip01Event _event(int index) {
  final author = (index + 1).toRadixString(16).padLeft(64, '0');
  final id = (index + 81).toRadixString(16).padLeft(64, '0');
  return Nip01Event(
    id: id,
    pubKey: author,
    kind: 22,
    content: 'Video $index',
    tags: const [
      ['imeta', 'url https://cdn.example/video.mp4', 'm video/mp4'],
    ],
  );
}
