import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('queries every NIP-71 video kind newest-first with metadata', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final metadatas = MockMetadatas();
    final older = Nip01Event(
      id: testEventId,
      pubKey: testCreatorPublicKey,
      kind: 21,
      tags: const [],
      content: '',
      createdAt: 10,
    );
    final newer = Nip01Event(
      id: secondTestEventId,
      pubKey: testCreatorPublicKey,
      kind: 34236,
      tags: const [],
      content: '',
      createdAt: 20,
    );
    when(() => ndk.requests).thenReturn(requests);
    when(() => ndk.metadata).thenReturn(metadatas);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse('videos', Stream.fromIterable([older, newer])));
    when(() => metadatas.loadMetadatas([testCreatorPublicKey], null))
        .thenAnswer(
      (_) async => [
        Metadata(pubKey: testCreatorPublicKey, displayName: 'Creator'),
      ],
    );
    final query = NdkNostrVideoEventQuery(ndk);

    final events = await query.loadVideoEvents(
      authorPublicKeys: {NostrPublicKeyHex.parse(testCreatorPublicKey)},
    );
    final publicKey = NostrPublicKeyHex.parse(testCreatorPublicKey);
    final metadata = await query.loadMetadataBatch({publicKey});

    expect(events.map((event) => event.id), [secondTestEventId, testEventId]);
    expect(metadata[publicKey]?.getName(), 'Creator');
    verify(() => metadatas.loadMetadatas([testCreatorPublicKey], null))
        .called(1);
    final call = verify(
      () => requests.query(
        name: 'ghostr-video-feed',
        filter: captureAny(named: 'filter'),
        timeout: const Duration(seconds: 5),
      ),
    );
    final filter = call.captured.single as Filter;
    expect(filter.kinds, [21, 22, 34235, 34236]);
    expect(filter.authors, [testCreatorPublicKey]);
  });
}
