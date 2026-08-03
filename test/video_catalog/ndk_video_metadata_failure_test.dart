import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

class _FailingMetadataQuery implements NostrVideoEventQueryPort {
  _FailingMetadataQuery(this.event);

  final Nip01Event event;

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
  }) async =>
      [event];

  @override
  Future<Map<NostrPublicKeyHex, Metadata>> loadMetadataBatch(
    Set<NostrPublicKeyHex> publicKeys,
  ) {
    throw StateError('metadata offline');
  }
}

void main() {
  test('keeps a valid video when creator metadata cannot be loaded', () async {
    final event = Nip01Event(
      id: testEventId,
      pubKey:
          '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e',
      kind: 22,
      content: 'Still playable',
      tags: const [
        ['imeta', 'url https://cdn.example/video.mp4', 'm video/mp4'],
      ],
    );

    final posts = await NdkVideoRemoteSource(
      _FailingMetadataQuery(event),
    ).loadRemoteFeed();

    expect(posts.single.id, testEventId);
    expect(posts.single.creator.displayName, startsWith('npub1'));
  });
}
