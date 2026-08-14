import 'package:flutter_test/flutter_test.dart';

import '../support/fake_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/hybrid_video_reader_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'Following older load falls back to the generic creator filter',
    () async {
      final older = samplePost(id: 'older');
      final remote = FakeRemoteVideoSource(const [])..olderPosts = [older];
      final fixture = hybridVideoReaderFixture(remote);
      final scope = testFollowingFeedScope();
      final cursor = DateTime.utc(2026, 1, 2);

      final posts = await fixture.reader.loadOlderFollowing(
        olderThan: cursor,
        scope: scope,
      );

      expect(posts, [older]);
      expect(remote.requestedCreatorIds, scope.creators);
      expect(remote.requestedOlderThan, [cursor]);
    },
  );
}
