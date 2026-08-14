import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('feed hydration has a short budget independent of mutations', () async {
    final repository = NostrVideoRepostRepository(
      _HangingReadClient(),
      relayHint: (_) async => 'wss://relay.example',
      hydrationTimeout: const Duration(milliseconds: 20),
      timeout: const Duration(seconds: 10),
    );

    final posts = await repository
        .hydrateAll([repostablePost()])
        .timeout(const Duration(milliseconds: 500));

    expect(posts.single.viewerHasReposted, isFalse);
    expect(
      posts.single.repostContext.observation,
      VideoRepostObservation.unobserved,
    );
  });
}

final class _HangingReadClient extends FakeNostrEventClient {
  _HangingReadClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> queries) {
    return Completer<List<NostrEventRecord>>().future;
  }
}
