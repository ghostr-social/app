import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/activity/presentation/activity_screen.dart';

import '../support/sample_data.dart';

void main() {
  testWidgets('shows an activity error and retries the request',
      (tester) async {
    final repository = _RetryingActivityRepository();
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: BlocProvider(
          create: (_) => ActivityCubit(repository)..load(),
          child: const ActivityScreen(),
        ),
      ),
    ));
    await tester.pumpAndSettle();

    expect(find.text('Activity unavailable'), findsOneWidget);
    expect(find.text('Could not load activity.'), findsOneWidget);

    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.text('Published a video'), findsOneWidget);
    expect(repository.loadCount, 2);
  });
}

class _RetryingActivityRepository implements ActivityRepository {
  int loadCount = 0;

  @override
  Future<List<ActivityItem>> load() async {
    loadCount += 1;
    if (loadCount == 1) throw const AppFailure('Could not load activity.');
    return [sampleActivity()];
  }

  @override
  Future<void> record(ActivityItem item) async {}
}
