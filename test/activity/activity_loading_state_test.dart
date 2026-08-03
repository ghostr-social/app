import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/activity/presentation/activity_screen.dart';

void main() {
  testWidgets('announces the activity loading state', (tester) async {
    final repository = _PendingActivityRepository();
    await tester.pumpWidget(MaterialApp(
      home: BlocProvider(
        create: (_) => ActivityCubit(repository)..load(),
        child: const ActivityScreen(),
      ),
    ));
    await tester.pump();

    expect(find.bySemanticsLabel('Loading activity'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
  });
}

class _PendingActivityRepository implements ActivityRepository {
  final _load = Completer<List<ActivityItem>>();

  @override
  ActivityRepository snapshotForActiveAccount() => this;

  @override
  Future<List<ActivityItem>> load() => _load.future;

  @override
  Future<void> record(ActivityItem item) async {}
}
