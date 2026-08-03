import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';

import '../support/sample_data.dart';

void main() {
  blocTest<ActivityCubit, ActivityState>(
    'emits a safe failure and reloads activity on retry',
    build: () => ActivityCubit(_RetryingRepository()),
    act: (cubit) async {
      await cubit.load();
      await cubit.load();
    },
    expect: () => [
      isA<ActivityLoading>(),
      isA<ActivityFailure>()
          .having((state) => state.message, 'message', 'Activity failed.'),
      isA<ActivityLoading>(),
      isA<ActivityLoaded>(),
    ],
  );
}

class _RetryingRepository implements ActivityRepository {
  int count = 0;

  @override
  ActivityRepository snapshotForActiveAccount() => this;

  @override
  Future<List<ActivityItem>> load() async {
    count += 1;
    if (count == 1) throw const AppFailure('Activity failed.');
    return [sampleActivity()];
  }

  @override
  Future<void> record(ActivityItem item) async {}
}
