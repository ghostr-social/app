import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_profile_search.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('translates an NDK profile search failure at the adapter boundary',
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
    ).thenThrow(StateError('socket closed'));
    final search = NdkNostrProfileSearch(ndk);

    await expectLater(
      search.searchProfiles('ali'),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'Could not search Nostr profiles.',
      )),
    );
  });
}
