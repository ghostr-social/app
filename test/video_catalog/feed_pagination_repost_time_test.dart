import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_pagination.dart';

import '../support/repost_samples.dart';

void main() {
  test('fresh feed cursor follows occurrence time rather than publication', () {
    final pagination = FeedPagination();

    pagination.restartFrom([repostedPost()]);

    expect(
      pagination.beginLoad()?.cursor,
      DateTime.utc(2026, 2, 1).subtract(const Duration(seconds: 1)),
    );
  });
}
