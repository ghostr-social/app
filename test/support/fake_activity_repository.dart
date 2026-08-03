import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';

class FakeActivityRepository implements AccountScopedActivityStore {
  FakeActivityRepository({List<ActivityItem>? items})
      : _items = items ?? <ActivityItem>[];

  final List<ActivityItem> _items;
  var activeAccountSnapshots = 0;

  @override
  ActivityRepository snapshotForActiveAccount() {
    activeAccountSnapshots += 1;
    return this;
  }

  @override
  AccountScopedActivityStore snapshotForAccount(NostrPublicKeyHex account) =>
      this;

  @override
  Future<List<ActivityItem>> load() async =>
      List<ActivityItem>.unmodifiable(_items);

  @override
  Future<void> record(ActivityItem item) async {
    _items.insert(0, item);
  }
}
