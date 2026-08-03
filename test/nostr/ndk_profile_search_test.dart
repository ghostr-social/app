import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_profile_search.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('profile text search hits search relays and keeps newest per author',
      () async {
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
    ).thenAnswer(
      (_) => NdkResponse(
        'p',
        Stream.fromIterable([
          _profileEvent(testViewerPublicKey, '{"name":"Old"}', 10),
          _profileEvent(testViewerPublicKey, '{"name":"New"}', 20),
          _profileEvent(testCreatorPublicKey, '{"name":"Other"}', 5),
        ]),
      ),
    );
    final search = NdkNostrProfileSearch(
      ndk,
      searchRelays: [RelayUrl.parse('wss://search.example')],
    );

    final profiles = await search.searchProfiles('ali');

    final captured = verify(
      () => requests.query(
        name: 'ghostr-profile-search',
        filter: captureAny(named: 'filter'),
        timeout: any(named: 'timeout'),
        explicitRelays: captureAny(named: 'explicitRelays'),
      ),
    ).captured;
    final filter = captured.first as Filter;
    expect(filter.kinds, [0]);
    expect(filter.search, 'ali');
    expect(filter.limit, 30);
    expect(captured.last, ['wss://search.example']);
    expect(profiles, hasLength(2));
    expect(
      profiles.firstWhere((p) => p.pubKey == testViewerPublicKey).name,
      'New',
    );
  });
}

Nip01Event _profileEvent(String pubKey, String content, int createdAt) {
  return Nip01Event(
    pubKey: pubKey,
    kind: 0,
    tags: const [],
    content: content,
    createdAt: createdAt,
  );
}
