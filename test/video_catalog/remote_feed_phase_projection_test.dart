import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

import '../support/discovery_search_fakes.dart';
import '../support/fake_remote_video_source.dart';

void main() {
  test(
    'remote feed phases keep their loading settled and failed meaning',
    () async {
      for (final expectation in <RemoteVideoPhase, VideoFeedUpdatePhase>{
        RemoteVideoPhase.loading: VideoFeedUpdatePhase.loading,
        RemoteVideoPhase.settled: VideoFeedUpdatePhase.settled,
        RemoteVideoPhase.failed: VideoFeedUpdatePhase.failed,
      }.entries) {
        final remote = FakeRemoteVideoSource([])
          ..snapshotPhase = expectation.key;
        final updates = RemoteVideoFeedUpdates(
          remote: remote,
          social: FakeSocialGraph(),
        );

        final actual = await updates.watchFeed(FeedKind.forYou).first;

        expect(actual.phase, expectation.value);
      }
    },
  );
}
