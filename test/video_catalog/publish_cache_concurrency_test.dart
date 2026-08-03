import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_nostr_video_publisher_port.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('concurrent successful publishes are both retained locally', () async {
    final store = _DelayedStore();
    final repository = HybridVideoPublishingRepository(
      store,
      FakeNostrVideoPublisherPort(),
      RecordingFailureReporter(),
    );

    final first = repository.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'First',
    );
    await store.firstSave.future;
    final second = repository.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Second',
    );
    await Future<void>.delayed(Duration.zero);
    store.release.complete();
    await Future.wait(<Future<VideoPublication>>[first, second]);

    expect(store.posts.map((post) => post.caption), {'First', 'Second'});
  });
}

class _DelayedStore implements PublishedVideoStore {
  final firstSave = Completer<void>();
  final release = Completer<void>();
  List<VideoPost> posts = <VideoPost>[];
  var saves = 0;

  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testViewerPublicKey);
  }

  @override
  PublishedVideoStore snapshotForActiveAccount() => this;

  @override
  Future<List<VideoPost>> loadPublishedPosts() async => List.of(posts);

  @override
  Future<void> savePublishedPosts(List<VideoPost> next) async {
    saves += 1;
    if (saves == 1) {
      firstSave.complete();
      await release.future;
    }
    posts = List.of(next);
  }
}
