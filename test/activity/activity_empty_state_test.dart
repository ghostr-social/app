import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/activity/presentation/activity_screen.dart';

import '../support/fake_activity_repository.dart';

void main() {
  testWidgets('renders the empty activity state', (tester) async {
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: BlocProvider(
          create: (_) => ActivityCubit(FakeActivityRepository())..load(),
          child: const ActivityScreen(),
        ),
      ),
    ));
    await tester.pumpAndSettle();

    expect(find.text('No activity yet'), findsOneWidget);
  });
}
