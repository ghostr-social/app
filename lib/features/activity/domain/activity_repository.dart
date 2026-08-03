import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';

abstract interface class ActivityRepository {
  ActivityRepository snapshotForActiveAccount();

  Future<List<ActivityItem>> load();

  Future<void> record(ActivityItem item);
}

abstract interface class AccountScopedActivityStore
    implements ActivityRepository {
  AccountScopedActivityStore snapshotForAccount(NostrPublicKeyHex account);
}
