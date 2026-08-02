import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';

void main() {
  test('rejects an empty activity identifier', () {
    expect(() => ActivityId.parse('  '), throwsFormatException);
  });
}
