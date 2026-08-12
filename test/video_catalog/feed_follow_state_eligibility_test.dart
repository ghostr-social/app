import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_follow_state.dart';

void main() {
  test('offers follow only for confirmed eligible creators', () {
    final viewer = ProfileId.parse('viewer');
    final followed = ProfileId.parse('followed');
    final available = ProfileId.parse('available');
    final unknown = FeedFollowState.unavailable(viewerId: viewer);

    expect(unknown.canFollow(available), isFalse);
    expect(unknown.accepted(available).canFollow(available), isFalse);

    final ready = FeedFollowState.ready(viewerId: viewer, followed: {followed});
    expect(ready.canFollow(viewer), isFalse);
    expect(ready.canFollow(followed), isFalse);
    expect(ready.canFollow(available), isTrue);

    final pending = ready.starting(available);
    expect(pending.canFollow(available), isFalse);
    expect(pending.rejected(available).canFollow(available), isTrue);
    expect(pending.accepted(available).canFollow(available), isFalse);
  });
}
