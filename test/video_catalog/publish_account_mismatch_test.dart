import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_nostr_video_publisher_port.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('rejects a cache snapshot owned by another publishing account',
      () async {
    final publisher = FakeNostrVideoPublisherPort();
    final repository = HybridVideoPublishingRepository(
      _OtherAccountStore(),
      publisher,
      RecordingFailureReporter(),
    );

    await expectLater(
      repository.publish(
        session: sampleSession(),
        media: sampleMedia(),
        caption: 'Do not publish',
      ),
      throwsA(isA<AppFailure>()),
    );

    expect(publisher.publishCount, 0);
  });
}

class _OtherAccountStore implements PublishedVideoStore {
  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testAuthorPublicKey);
  }

  @override
  PublishedVideoStore snapshotForActiveAccount() => this;

  @override
  Future<List<VideoPost>> loadPublishedPosts() async => const <VideoPost>[];

  @override
  Future<void> savePublishedPosts(List<VideoPost> posts) async {}
}
