import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';

void main() {
  test('uses an app-safe message for an unexpected activity load error',
      () async {
    final cubit = ActivityCubit(_UnexpectedActivityRepository());
    addTearDown(cubit.close);

    await cubit.load();

    expect(
      (cubit.state as ActivityFailure).message,
      'Could not load activity. Try again.',
    );
  });
}

class _UnexpectedActivityRepository implements ActivityRepository {
  @override
  Future<List<ActivityItem>> load() => throw StateError('database unavailable');

  @override
  Future<void> record(ActivityItem item) async {}
}
