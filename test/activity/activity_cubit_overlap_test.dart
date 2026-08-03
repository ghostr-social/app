import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('an older activity load cannot replace a newer result', () async {
    final repository = _OverlappingActivityRepository();
    final cubit = ActivityCubit(repository);
    addTearDown(cubit.close);

    final first = cubit.load();
    final second = cubit.load();
    repository.second.complete(<ActivityItem>[_item('newer')]);
    await second;
    repository.first.complete(<ActivityItem>[_item('older')]);
    await first;

    expect((cubit.state as ActivityLoaded).items.single.id.value, 'newer');
  });
}

ActivityItem _item(String id) {
  final item = sampleActivity();
  return ActivityItem(
    id: ActivityId.parse(id),
    type: item.type,
    description: item.description,
    occurredAt: item.occurredAt,
  );
}

class _OverlappingActivityRepository implements ActivityRepository {
  final first = Completer<List<ActivityItem>>();
  final second = Completer<List<ActivityItem>>();
  var calls = 0;

  @override
  ActivityRepository snapshotForActiveAccount() => this;

  @override
  Future<List<ActivityItem>> load() {
    return calls++ == 0 ? first.future : second.future;
  }

  @override
  Future<void> record(ActivityItem item) async {}
}
