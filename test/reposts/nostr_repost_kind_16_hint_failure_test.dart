import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('generic repost publishes when optional relay lookup fails', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) => throw const AppFailure('relay list unavailable'),
    );

    await repository.toggleRepost(repostablePost(kind: 22));

    final event = client.events.single;
    expect(event.kind.value, 16);
    expect(event.tags.toRaw().first, ['e', testEventId]);
  });
}
