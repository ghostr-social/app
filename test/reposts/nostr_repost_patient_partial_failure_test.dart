import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test(
    'patient hydration keeps completed chunks after a later failure',
    () async {
      final client = _FailingFifthBatchClient();
      final repository = NostrVideoRepostRepository(
        client,
        relayHint: (_) async => 'wss://relay.example',
      );
      final posts = [
        for (var index = 1; index <= 81; index += 1)
          repostablePost(eventId: index.toRadixString(16).padLeft(64, '0')),
      ];

      final hydrated = await repository.hydrateAll(
        posts,
        mode: VideoRepostHydration.patient,
      );

      expect(hydrated.take(80).every(_observed), isTrue);
      expect(_observed(hydrated.last), isFalse);
    },
  );
}

bool _observed(VideoPost post) {
  return post.repostContext.observation == VideoRepostObservation.observed;
}

final class _FailingFifthBatchClient extends FakeNostrEventClient {
  _FailingFifthBatchClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> queries) {
    if (requestCount == 4) throw const AppFailure('offline');
    return super.queryBatch(queries);
  }
}
