import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('hashtag queries target search relays merged with the outbox', () async {
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
    ).thenAnswer((_) => NdkResponse('t', const Stream<Nip01Event>.empty()));
    final query = NdkNostrVideoEventQuery(
      ndk,
      searchRelays: [RelayUrl.parse('wss://search.example')],
      outbox: _StubOutbox(ndk),
    );

    await query.loadVideoEvents(hashtags: {'dance'});

    final captured = verify(
      () => requests.query(
        name: 'ghostr-video-search',
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: captureAny(named: 'explicitRelays'),
      ),
    ).captured;
    expect(
      captured.single,
      unorderedEquals(['wss://search.example', 'wss://follows.example']),
    );
  });
}

class _StubOutbox extends NdkNostrOutboxDirectory {
  _StubOutbox(super.ndk);

  @override
  Future<List<String>> discoveryRelayUrls() async {
    return const ['wss://follows.example'];
  }

  @override
  Future<List<String>> authorWriteRelayUrls(
    Set<NostrPublicKeyHex> authors,
  ) async {
    return const ['wss://creator.example'];
  }
}
