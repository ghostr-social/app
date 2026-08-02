import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';

class ActivityItemStorageMapper {
  const ActivityItemStorageMapper();

  ActivityItem fromMap(Map<String, dynamic> map) {
    return ActivityItem(
      id: ActivityId.parse(_requiredString(map, 'id')),
      type: _activityType(map),
      description: ActivityDescription(
        title: _requiredString(map, 'title'),
        body: _requiredString(map, 'body'),
      ),
      occurredAt: _requiredDateTime(map, 'occurredAt'),
    );
  }

  Map<String, Object?> toMap(ActivityItem item) {
    return <String, Object?>{
      'id': item.id,
      'type': item.type.name,
      'title': item.title,
      'body': item.body,
      'occurredAt': item.occurredAt.toIso8601String(),
    };
  }

  ActivityType _activityType(Map<String, dynamic> map) {
    final name = _requiredString(map, 'type');
    try {
      return ActivityType.values.byName(name);
    } on ArgumentError {
      throw FormatException('Invalid activity type: $name');
    }
  }

  String _requiredString(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value is String) return value;
    throw FormatException('Activity field "$key" must be a string.');
  }

  DateTime _requiredDateTime(Map<String, dynamic> map, String key) {
    final value = _requiredString(map, key);
    return DateTime.parse(value);
  }
}
