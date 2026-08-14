import 'package:flutter_test/flutter_test.dart';

import '../support/fake_following_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/hybrid_video_reader_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test('Following load uses the account-scoped remote operation', () async {
    final remote = FakeFollowingRemoteVideoSource(
      followingPosts: [samplePost(id: 'following')],
    );
    final fixture = hybridVideoReaderFixture(remote);
    final scope = testFollowingFeedScope();

    final posts = await fixture.reader.loadFollowing(scope);

    expect(posts.single.id.value, 'following');
    expect(remote.requestedFollowingScope?.sameAs(scope), isTrue);
    expect(remote.loadCount, 0);
  });
}
