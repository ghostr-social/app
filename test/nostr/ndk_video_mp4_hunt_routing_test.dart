import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('the feed mp4 hunt runs on the search relays, not the outbox', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenAnswer((_) => NdkResponse('p', const Stream<Nip01Event>.empty()));
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: any(named: 'explicitRelays'),
      ),
    ).thenAnswer((_) => NdkResponse('s', const Stream<Nip01Event>.empty()));
    final query = NdkNostrVideoEventQuery(
      ndk,
      searchRelays: [RelayUrl.parse('wss://search.example')],
    );

    await query.loadVideoEvents();

    final captured = verify(
      () => requests.query(
        name: 'ghostr-note-search',
        filter: captureAny(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: captureAny(named: 'explicitRelays'),
      ),
    ).captured;
    expect((captured.first as Filter).search, 'mp4');
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
