import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('patient chunks never return across an account change', () async {
    final client = _SwitchingFifthBatchClient();
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );
    final posts = [
      for (var index = 1; index <= 81; index += 1)
        repostablePost(eventId: index.toRadixString(16).padLeft(64, '0')),
    ];

    final hydration = repository.hydrateAll(
      posts,
      mode: VideoRepostHydration.patient,
    );

    await expectLater(
      hydration,
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'The active account changed. Try again.',
        ),
      ),
    );
  });
}

final class _SwitchingFifthBatchClient extends FakeNostrEventClient {
  _SwitchingFifthBatchClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> queries) {
    if (requestCount == 4) {
      publicKeyHex = NostrPublicKeyHex.parse(testCreatorPublicKey);
      throw const AppFailure('offline');
    }
    return super.queryBatch(queries);
  }
}
