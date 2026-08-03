import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('drops video results outside the requested kinds and authors', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final matching = _event(1, testAuthorPublicKey, 21);
    final returned = <Nip01Event>[
      _event(2, testAuthorPublicKey, 7),
      _event(3, testCreatorPublicKey, 21),
      matching,
    ];
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse('videos', Stream.fromIterable(returned)));
    final query = NdkNostrVideoEventQuery(ndk);

    final result = await query.loadVideoEvents(
      authorPublicKeys: <NostrPublicKeyHex>{
        NostrPublicKeyHex.parse(testAuthorPublicKey),
      },
    );

    expect(result.map((event) => event.id), <String>[matching.id]);
  });
}

Nip01Event _event(int sequence, String author, int kind) {
  return Nip01Event(
    id: publishedEventId(sequence),
    pubKey: author,
    kind: kind,
    tags: const <List<String>>[],
    content: '',
    createdAt: 10,
  );
}
