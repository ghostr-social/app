import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_screen.dart';

import '../support/fakes.dart';

void main() {
  testWidgets('clearing watch history empties the list', (tester) async {
    final repository = FakeWatchHistoryRepository(entries: [
      WatchHistoryEntry(
        videoId: 'e:video-1',
        title: 'A relay-side banger',
        creatorName: 'Nora Relay',
        watchedAt: DateTime(2026, 3, 12, 10, 30),
      ),
    ]);

    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => WatchHistoryCubit(repository)..load(),
          child: const WatchHistoryScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Clear watch history'));
    await tester.pumpAndSettle();

    expect(find.text('No watched videos yet'), findsOneWidget);
    expect(repository.entries, isEmpty);
  });
}
