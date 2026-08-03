import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/hybrid_video_reader.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_nostr_comments_port.dart';
import '../support/fake_nostr_engagement_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('loads canonical videos when the local projection cannot be read',
      () async {
    final reporter = RecordingFailureReporter();
    final canonical = samplePost(caption: 'Available from relays');
    final reader = HybridVideoReader(
      remote: FakeRemoteVideoSource([canonical]),
      local: _FailingPublishedVideoStore(),
      interactions: NostrVideoInteractions(
        FakeNostrEngagementPort(),
        FakeNostrCommentsPort(),
        reporter,
      ),
      failureReporter: reporter,
    );

    final posts = await reader.load();

    expect(posts.single.caption, 'Available from relays');
    expect(reporter.sources, contains('HybridVideoReader.loadLocal'));
  });
}

class _FailingPublishedVideoStore implements PublishedVideoStore {
  @override
  NostrPublicKeyHex get accountPublicKey =>
      NostrPublicKeyHex.parse(testViewerPublicKey);

  @override
  PublishedVideoStore snapshotForActiveAccount() => this;

  @override
  Future<List<VideoPost>> loadPublishedPosts() {
    throw const AppFailure('Local projection unavailable.');
  }

  @override
  Future<void> savePublishedPosts(List<VideoPost> posts) async {}
}
