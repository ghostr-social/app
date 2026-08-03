import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_nostr_video_publisher_port.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('publishes remotely even when the local cache cannot be read', () async {
    final publisher = FakeNostrVideoPublisherPort();
    final reporter = RecordingFailureReporter();
    final repository = HybridVideoPublishingRepository(
      _UnreadableStore(),
      publisher,
      reporter,
    );

    final publication = await repository.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Public despite cache',
    );

    expect(publication.post.caption, 'Public despite cache');
    expect(
      publication.cacheStatus,
      VideoPublicationCacheStatus.unavailable,
    );
    expect(publisher.publishCount, 1);
    expect(reporter.sources, [
      'HybridVideoPublishingRepository.savePublishedPosts',
    ]);
  });
}

class _UnreadableStore implements PublishedVideoStore {
  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testViewerPublicKey);
  }

  @override
  PublishedVideoStore snapshotForActiveAccount() => this;

  @override
  Future<List<VideoPost>> loadPublishedPosts() {
    throw StateError('corrupt cache');
  }

  @override
  Future<void> savePublishedPosts(List<VideoPost> posts) async {}
}
