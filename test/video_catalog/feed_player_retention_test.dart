import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_navigation_history.dart';
import 'package:ghostr/features/video_catalog/domain/feed_player_retention.dart';

void main() {
  const retention = FeedPlayerRetention(
    maximumControllers: 8,
    minimumPrevious: 2,
    history: FeedNavigationHistory.ordinary,
  );

  test('spare capacity keeps the complete bounded back history warm', () {
    expect(retention.warmPreviousDepth(preparedFutureCount: 2), 3);
  });

  test('prepared future players retain priority over the back history', () {
    expect(retention.warmPreviousDepth(preparedFutureCount: 5), 2);
    expect(retention.warmPreviousDepth(preparedFutureCount: 7), 0);
  });

  test('one rolling replenishment slot is reserved below future capacity', () {
    expect(
      retention.warmPreviousDepth(preparedFutureCount: 4, canReplenish: true),
      2,
    );
    expect(
      retention.warmPreviousDepth(preparedFutureCount: 2, canReplenish: true),
      3,
    );
  });
}
