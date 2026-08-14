import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('accepted wrapper is not applied after the account changes', () async {
    final client = _SwitchingClient();
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );

    await expectLater(
      repository.toggleRepost(repostablePost()),
      throwsA(isA<AppFailure>()),
    );
    expect(client.events.single.kind.value, 6);
  });
}

final class _SwitchingClient extends FakeNostrEventClient {
  _SwitchingClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    final id = await super.publish(event, expectedAuthor: expectedAuthor);
    publicKeyHex = NostrPublicKeyHex.parse(testAuthorPublicKey);
    return id;
  }
}
