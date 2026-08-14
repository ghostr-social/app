import 'package:flutter_test/flutter_test.dart';

import '../support/fake_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/hybrid_video_reader_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test('generic Following fallback reads local posts once', () async {
    final local = samplePost(id: 'local');
    final fixture = hybridVideoReaderFixture(
      FakeRemoteVideoSource(const []),
      localPosts: [local],
    );

    final posts = await fixture.reader.loadFollowing(testFollowingFeedScope());

    expect(posts, [local]);
    expect(fixture.local.loadCount, 1);
  });
}
