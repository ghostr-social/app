import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/hybrid_video_reader.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import 'fake_nostr_comments_port.dart';
import 'fake_nostr_engagement_port.dart';
import 'nostr_test_values.dart';
import 'recording_failure_reporter.dart';

final class HybridVideoReaderFixture {
  const HybridVideoReaderFixture({
    required this.reader,
    required this.local,
    required this.reporter,
  });

  final HybridVideoReader reader;
  final MemoryPublishedVideoStore local;
  final RecordingFailureReporter reporter;
}

HybridVideoReaderFixture hybridVideoReaderFixture(
  RemoteVideoSource remote, {
  List<VideoPost> localPosts = const <VideoPost>[],
}) {
  final local = MemoryPublishedVideoStore(localPosts);
  final reporter = RecordingFailureReporter();
  return HybridVideoReaderFixture(
    reader: HybridVideoReader(
      remote: remote,
      local: local,
      interactions: NostrVideoInteractions(
        FakeNostrEngagementPort(),
        FakeNostrCommentsPort(),
        reporter,
      ),
      failureReporter: reporter,
    ),
    local: local,
    reporter: reporter,
  );
}

final class MemoryPublishedVideoStore implements PublishedVideoStore {
  MemoryPublishedVideoStore(this.posts);

  List<VideoPost> posts;
  int loadCount = 0;

  @override
  NostrPublicKeyHex get accountPublicKey =>
      NostrPublicKeyHex.parse(testViewerPublicKey);

  @override
  Future<List<VideoPost>> loadPublishedPosts() async {
    loadCount += 1;
    return posts;
  }

  @override
  Future<void> savePublishedPosts(List<VideoPost> posts) async {
    this.posts = posts;
  }

  @override
  PublishedVideoStore snapshotForActiveAccount() => this;
}
