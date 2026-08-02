import 'package:ghostr/features/activity/domain/activity_type.dart';

extension type const ActivityId._(String value) implements String {
  factory ActivityId.parse(String raw) {
    final value = raw.trim();
    if (value.isEmpty) throw const FormatException('Invalid activity ID.');
    return ActivityId._(value);
  }
}

class ActivityDescription {
  factory ActivityDescription({required String title, required String body}) {
    final cleanTitle = title.trim();
    final cleanBody = body.trim();
    if (cleanTitle.isEmpty || cleanBody.isEmpty) {
      throw const FormatException('Activity text cannot be empty.');
    }
    return ActivityDescription._(cleanTitle, cleanBody);
  }

  const ActivityDescription._(this.title, this.body);

  final String title;
  final String body;
}

class ActivityItem {
  const ActivityItem({
    required this.id,
    required this.type,
    required this.description,
    required this.occurredAt,
  });

  final ActivityId id;
  final ActivityType type;
  final ActivityDescription description;
  final DateTime occurredAt;

  String get title => description.title;
  String get body => description.body;
}
