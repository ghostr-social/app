import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a search fans out to a note query and merges both results', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    when(() => ndk.requests).thenReturn(requests);
    final video = Nip01Event(
      id: testEventId,
      pubKey: testCreatorPublicKey,
      kind: 22,
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
        name: 'ghostr-video-search',
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: any(named: 'explicitRelays'),
      ),
    ).thenAnswer((_) => NdkResponse('v', Stream.fromIterable([video])));
    when(
      () => requests.query(
        name: 'ghostr-note-search',
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: any(named: 'explicitRelays'),
      ),
    ).thenAnswer((_) => NdkResponse('n', Stream.fromIterable([note])));
    final query = NdkNostrVideoEventQuery(
      ndk,
      searchRelays: [RelayUrl.parse('wss://search.example')],
    );

    final events = await query.loadVideoEvents(searchQuery: 'clip');

    expect(events.map((event) => event.id), [secondTestEventId, testEventId]);
    final noteFilter = verify(
      () => requests.query(
        name: 'ghostr-note-search',
        filter: captureAny(named: 'filter'),
        timeout: const Duration(seconds: 8),
        explicitRelays: ['wss://search.example'],
      ),
    ).captured.single as Filter;
    expect(noteFilter.kinds, [1]);
    expect(noteFilter.search, 'clip');
  });
}
