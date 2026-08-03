import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('requests hashtag-tagged videos with a widened limit', () async {
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
      (_) => NdkResponse('videos', const Stream<Nip01Event>.empty()),
    );
    final query = NdkNostrVideoEventQuery(ndk);

    await query.loadVideoEvents(hashtags: {'dance'});
    await query.loadVideoEvents();

    final captured = verify(
      () => requests.query(
        name: 'ghostr-video-feed',
        filter: captureAny(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).captured;
    final hashtagFilter = captured.first as Filter;
    final unscopedFilter = captured.last as Filter;
    expect(hashtagFilter.tags?.keys, ['#t']);
    expect(
      hashtagFilter.tags?['#t'],
      unorderedEquals(<String>['dance', 'Dance', 'DANCE']),
    );
    expect(hashtagFilter.limit, 200);
    expect(unscopedFilter.tTags, isNull);
    expect(unscopedFilter.limit, 80);
  });
}
