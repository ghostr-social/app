import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

class _FakeEventQuery implements NostrVideoEventQueryPort {
  _FakeEventQuery(this.events, this.metadata);

  final List<Nip01Event> events;
  final Metadata metadata;
  Set<NostrPublicKeyHex>? requestedAuthorPublicKeys;

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
  }) async {
    requestedAuthorPublicKeys = authorPublicKeys;
    return events;
  }

  @override
  Future<Metadata?> loadMetadata(NostrPublicKeyHex publicKey) async => metadata;
}

void main() {
  test('loads valid NIP-71 videos and skips malformed relay events', () async {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final valid = Nip01Event(
      id: testEventId,
      pubKey: publicKey,
      kind: 22,
      content: 'Playable',
      tags: const [
        ['imeta', 'url https://cdn.example/video.mp4', 'm video/mp4'],
      ],
    );
    final malformed = Nip01Event(
      id: secondTestEventId,
      pubKey: publicKey,
      kind: 22,
      content: 'Missing media',
      tags: const [],
    );
    final query = _FakeEventQuery(
      [valid, malformed],
      Metadata(pubKey: publicKey, displayName: 'Nora'),
    );
    final source = NdkVideoRemoteSource(query);

    final posts = await source.loadRemoteFeed(
      creatorIds: {ProfileId.parse(Nip19.encodePubKey(publicKey))},
    );

    expect(posts.map((post) => post.id), [testEventId]);
    expect(query.requestedAuthorPublicKeys, {
      NostrPublicKeyHex.parse(publicKey),
    });
  });
}
