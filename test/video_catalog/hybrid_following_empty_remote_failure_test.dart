import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fake_following_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/hybrid_video_reader_fixture.dart';

void main() {
  test('typed Following failure is rethrown without local posts', () async {
    const failure = AppFailure('relays unavailable');
    final remote = FakeFollowingRemoteVideoSource()..followingFailure = failure;
    final fixture = hybridVideoReaderFixture(remote);

    await expectLater(
      fixture.reader.loadFollowing(testFollowingFeedScope()),
      throwsA(same(failure)),
    );
    expect(fixture.reporter.sources, ['HybridVideoReader.loadFollowing']);
  });
}
