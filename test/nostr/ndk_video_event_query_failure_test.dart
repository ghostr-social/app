import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('translates an NDK video query failure at the adapter boundary',
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
    ).thenThrow(StateError('relay socket failed'));
    final query = NdkNostrVideoEventQuery(ndk);

    await expectLater(
      query.loadVideoEvents(),
      throwsA(isA<AppFailure>()),
    );
  });
}
