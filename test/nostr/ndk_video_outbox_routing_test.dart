import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('feed queries route to outbox relays by scope', () async {
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
    ).thenAnswer((_) => NdkResponse('f', const Stream<Nip01Event>.empty()));
    final query = NdkNostrVideoEventQuery(ndk, outbox: _StubOutbox(ndk));

    await query.loadVideoEvents(hashtags: {'dance'});
    await query.loadVideoEvents(
      authorPublicKeys: {NostrPublicKeyHex.parse(testCreatorPublicKey)},
    );

    final captured = verify(
      () => requests.query(
        name: 'ghostr-video-feed',
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: captureAny(named: 'explicitRelays'),
      ),
    ).captured;
    expect(captured.first, ['wss://follows.example']);
    expect(captured.last, ['wss://creator.example']);
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
