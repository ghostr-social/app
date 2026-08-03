import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('an older-than cutoff reaches the relay filter and is revalidated',
      () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final olderThan = DateTime.utc(2026, 8, 1, 12);
    final cutoff = olderThan.millisecondsSinceEpoch ~/ 1000;
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse(
      'videos',
      Stream.fromIterable([
        _event(1, createdAt: cutoff - 10),
        _event(2, createdAt: cutoff + 10),
      ]),
    ));

    final result = await NdkNostrVideoEventQuery(ndk)
        .loadVideoEvents(olderThan: olderThan);

    final filter = verify(
      () => requests.query(
        name: any(named: 'name'),
        filter: captureAny(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).captured.single as Filter;
    expect(filter.until, cutoff);
    expect(result.map((event) => event.id), [publishedEventId(1)]);
  });
}

Nip01Event _event(int sequence, {required int createdAt}) {
  return Nip01Event(
    id: publishedEventId(sequence),
    pubKey: testAuthorPublicKey,
    kind: 21,
    tags: const <List<String>>[],
    content: '',
    createdAt: createdAt,
  );
}
