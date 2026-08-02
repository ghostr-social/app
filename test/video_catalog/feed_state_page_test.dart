import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';

import '../support/sample_data.dart';

void main() {
  test('changes the active feed page without dropping its notice', () {
    final state = FeedLoaded(
      FeedKind.following,
      [samplePost(), samplePost(id: 'post-2'), samplePost(id: 'post-3')],
      notice: 'Relay synchronized.',
    );

    final changed = state.withPage(2);

    expect(changed.activeIndex, 2);
    expect(changed.kind, FeedKind.following);
    expect(changed.notice, 'Relay synchronized.');
  });
}
