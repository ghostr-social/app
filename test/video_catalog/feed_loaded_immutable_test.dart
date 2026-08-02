import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';

import '../support/sample_data.dart';

void main() {
  test('does not expose a mutable loaded-feed collection', () {
    final state = FeedLoaded(FeedKind.forYou, [samplePost()]);

    expect(() => state.posts.clear(), throwsUnsupportedError);
  });
}
