import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('the plain feed also hunts kind-1 notes and merges them', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    when(() => ndk.requests).thenReturn(requests);
    final video = Nip01Event(
      id: testEventId,
      pubKey: testCreatorPublicKey,
      kind: 21,
      tags: const [],
      content: '',
      createdAt: 10,
    );
    final note = Nip01Event(
      id: secondTestEventId,
      pubKey: testCreatorPublicKey,
      kind: 1,
      tags: const [],
      content: 'https://cdn.example/clip.mp4',
      createdAt: 20,
    );
    when(
      () => requests.query(
        name: 'ghostr-video-feed',
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenAnswer((_) => NdkResponse('v', Stream.fromIterable([video])));
    when(
      () => requests.query(
        name: 'ghostr-note-feed',
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenAnswer((_) => NdkResponse('n', Stream.fromIterable([note])));
    final query = NdkNostrVideoEventQuery(ndk);

    final events = await query.loadVideoEvents();

    expect(events.map((event) => event.id), [secondTestEventId, testEventId]);
    final noteFilter = verify(
      () => requests.query(
        name: 'ghostr-note-feed',
        filter: captureAny(named: 'filter'),
        timeout: const Duration(seconds: 5),
      ),
    ).captured.single as Filter;
    expect(noteFilter.kinds, [1]);
    expect(noteFilter.limit, 200);
  });
}
