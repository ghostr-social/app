import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

void main() {
  test('data usage levels bound how many requests may run at once', () {
    expect(DataUsageLevel.conservative.maxConcurrentRequests, 2);
    expect(DataUsageLevel.balanced.maxConcurrentRequests, 4);
    expect(DataUsageLevel.aggressive.maxConcurrentRequests, 6);
    expect(DataUsageLevel.balanced.label, 'Balanced');
    for (final level in DataUsageLevel.values) {
      expect(level.maxConcurrentRequests, greaterThan(0));
    }
  });
}
