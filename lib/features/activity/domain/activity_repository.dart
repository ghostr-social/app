import 'package:ghostr/features/activity/domain/activity_item.dart';

abstract interface class ActivityRepository {
  Future<List<ActivityItem>> load();

  Future<void> record(ActivityItem item);
}
