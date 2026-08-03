import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

void main() {
  test('data usage levels bound how many outbox relays are queried', () {
    expect(DataUsageLevel.conservative.maxOutboxRelays, 6);
    expect(DataUsageLevel.balanced.maxOutboxRelays, 12);
    expect(DataUsageLevel.aggressive.maxOutboxRelays, 18);
    expect(
      DataUsageLevel.conservative.maxOutboxRelays <
          DataUsageLevel.aggressive.maxOutboxRelays,
      isTrue,
    );
  });
}
