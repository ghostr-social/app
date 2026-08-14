import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('patient hydration corrects a repost after the prompt budget', () async {
    final client = _DelayedClient();
    client.events.add(_wrapper());
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
      hydrationTimeout: const Duration(milliseconds: 5),
    );

    final prompt = (await repository.hydrateAll([repostablePost()])).single;
    final patient = (await repository.hydrateAll([
      prompt,
    ], mode: VideoRepostHydration.patient)).single;

    expect(prompt.repostContext.observation, VideoRepostObservation.unobserved);
    expect(patient.viewerHasReposted, isTrue);
    expect(patient.repostContext.observation, VideoRepostObservation.observed);
  });
}

final class _DelayedClient extends FakeNostrEventClient {
  _DelayedClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> queries,
  ) async {
    await Future<void>.delayed(const Duration(milliseconds: 20));
    return super.queryBatch(queries);
  }
}

NostrEventRecord _wrapper() =>
    NostrUnsignedEvent(
      kind: 6,
      tags: const [
        ['e', testEventId],
      ],
      content: '',
    ).toRecord(
      id: secondTestEventId,
      authorPublicKeyHex: testViewerPublicKey,
      createdAt: 10,
    );
