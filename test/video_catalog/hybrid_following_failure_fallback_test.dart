import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fake_following_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/hybrid_video_reader_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'Following load returns local posts when its scoped read fails',
    () async {
      final remote = FakeFollowingRemoteVideoSource()
        ..followingFailure = const AppFailure('relays unavailable');
      final local = samplePost(id: 'local');
      final fixture = hybridVideoReaderFixture(remote, localPosts: [local]);

      final posts = await fixture.reader.loadFollowing(
        testFollowingFeedScope(),
      );

      expect(posts, [local]);
      expect(
        fixture.reporter.sources,
        contains('HybridVideoReader.loadFollowing'),
      );
    },
  );
}
