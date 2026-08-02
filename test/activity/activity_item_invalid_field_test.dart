import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/data/activity_item_storage_mapper.dart';

void main() {
  test('rejects a persisted activity field with the wrong type', () {
    expect(
      () => const ActivityItemStorageMapper().fromMap(const {'id': 1}),
      throwsFormatException,
    );
  });
}
