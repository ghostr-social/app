import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('returns newest 80 unique videos with stable tie order', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final videos = List<Nip01Event>.generate(82, (index) {
      final sequence = index + 1;
      return _event(sequence, createdAt: sequence > 80 ? 82 : sequence);
    })
      ..insert(20, _event(82));
    when(() => ndk.requests).thenReturn(requests);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        timeout: any(named: 'timeout'),
      ),
    ).thenReturn(NdkResponse('videos', Stream.fromIterable(videos)));

    final result = await NdkNostrVideoEventQuery(ndk).loadVideoEvents();

    expect(result, hasLength(80));
    expect(
      result.map((event) => event.id),
      List<String>.generate(80, (index) => publishedEventId(82 - index)),
    );
    expect(result.map((event) => event.id).toSet(), hasLength(80));
  });
}

Nip01Event _event(int sequence, {int? createdAt}) {
  return Nip01Event(
    id: publishedEventId(sequence),
    pubKey: testAuthorPublicKey,
    kind: 21,
    tags: const <List<String>>[],
    content: '',
    createdAt: createdAt ?? sequence,
  );
}
