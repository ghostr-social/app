import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_screen.dart';

import '../support/fakes.dart';

void main() {
  testWidgets('shows the empty panel when no videos were watched',
      (tester) async {
    final repository = FakeWatchHistoryRepository();

    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => WatchHistoryCubit(repository)..load(),
          child: const WatchHistoryScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('No watched videos yet'), findsOneWidget);
  });
}
