import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('search queries target the NIP-50 search relays explicitly', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: any(named: 'explicitRelays'),
      ),
    ).thenAnswer((_) => NdkResponse('s', const Stream<Nip01Event>.empty()));
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenAnswer((_) => NdkResponse('f', const Stream<Nip01Event>.empty()));
    final query = NdkNostrVideoEventQuery(
      ndk,
      searchRelays: [RelayUrl.parse('wss://search.example')],
    );

    await query.loadVideoEvents(searchQuery: 'ghost dance');
    await query.loadVideoEvents();

    final captured = verify(
      () => requests.query(
        name: 'ghostr-video-search',
        filter: captureAny(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: captureAny(named: 'explicitRelays'),
      ),
    ).captured;
    expect((captured.first as Filter).search, 'ghost dance');
    expect(captured.last, ['wss://search.example']);
    verify(
      () => requests.query(
        name: 'ghostr-video-feed',
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).called(1);
  });
}
