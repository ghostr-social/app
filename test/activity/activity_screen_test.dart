import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/activity/presentation/activity_screen.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('renders activity items', (tester) async {
    final populated = FakeActivityRepository(
      items: [sampleActivity()],
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BlocProvider(
            create: (_) => ActivityCubit(populated)..load(),
            child: const ActivityScreen(),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Published a video'), findsOneWidget);
  });
}
