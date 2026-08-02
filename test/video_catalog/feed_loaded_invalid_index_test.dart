import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';

import '../support/sample_data.dart';

void main() {
  test('rejects an out-of-range loaded-feed index', () {
    expect(
      () => FeedLoaded(FeedKind.forYou, [samplePost()], activeIndex: 1),
      throwsRangeError,
    );
  });
}
