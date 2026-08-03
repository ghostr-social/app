import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a failing note query never sinks the video search results', () async {
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
    ).thenAnswer(
      (_) => NdkResponse('n', Stream<Nip01Event>.error(StateError('down'))),
    );
    final query = NdkNostrVideoEventQuery(
      ndk,
      searchRelays: [RelayUrl.parse('wss://search.example')],
    );

    final events = await query.loadVideoEvents(searchQuery: 'clip');

    expect(events.map((event) => event.id), [testEventId]);
  });
}
