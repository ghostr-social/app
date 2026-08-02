import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/data/activity_item_storage_mapper.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';

void main() {
  test('serializes and deserializes activity items', () {
    const mapper = ActivityItemStorageMapper();
    final item = ActivityItem(
      id: ActivityId.parse('activity-1'),
      type: ActivityType.publish,
      description: ActivityDescription(
        title: 'Published a video',
        body: 'Ghostr clip',
      ),
      occurredAt: DateTime(2026, 3, 12, 10),
    );

    final decoded = mapper.fromMap(mapper.toMap(item));

    expect(decoded.title, item.title);
    expect(decoded.type, item.type);
  });
}
