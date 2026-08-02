import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';

class FakeActivityRepository implements ActivityRepository {
  FakeActivityRepository({List<ActivityItem>? items})
      : _items = items ?? <ActivityItem>[];

  final List<ActivityItem> _items;

  @override
  Future<List<ActivityItem>> load() async =>
      List<ActivityItem>.unmodifiable(_items);

  @override
  Future<void> record(ActivityItem item) async {
    _items.insert(0, item);
  }
}
