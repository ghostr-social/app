import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';

void main() {
  test('ignores an activity load completion after disposal', () async {
    final repository = _PendingActivityRepository();
    final cubit = ActivityCubit(repository);

    final load = cubit.load();
    final completion = expectLater(load, completes);
    await cubit.close();
    repository.pending.complete(const []);

    await completion;
  });
}

class _PendingActivityRepository implements ActivityRepository {
  final pending = Completer<List<ActivityItem>>();

  @override
  Future<List<ActivityItem>> load() => pending.future;

  @override
  Future<void> record(ActivityItem item) async {}
}
