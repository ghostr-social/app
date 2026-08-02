import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/data/activity_item_storage_mapper.dart';

void main() {
  test('rejects an unknown persisted activity type', () {
    expect(
      () => const ActivityItemStorageMapper().fromMap(const {
        'id': 'activity-1',
        'type': 'unknown',
        'title': 'Title',
        'body': 'Body',
        'occurredAt': '2026-08-02T12:00:00.000Z',
      }),
      throwsFormatException,
    );
  });
}
