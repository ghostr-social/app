import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_profile_search.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a pasted npub resolves as a direct author lookup, not text search',
      () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenAnswer(
      (_) => NdkResponse(
        'p',
        Stream.fromIterable([
          Nip01Event(
            pubKey: testViewerPublicKey,
            kind: 0,
            tags: const [],
            content: '{"name":"Alice"}',
          ),
        ]),
      ),
    );
    final search = NdkNostrProfileSearch(
      ndk,
      searchRelays: [RelayUrl.parse('wss://search.example')],
    );

    final profiles = await search.searchProfiles(testViewerNpub);

    final filter = verify(
      () => requests.query(
        name: 'ghostr-profile-search',
        filter: captureAny(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).captured.single as Filter;
    expect(filter.authors, [testViewerPublicKey]);
    expect(filter.search, isNull);
    expect(profiles.single.name, 'Alice');
  });
}
