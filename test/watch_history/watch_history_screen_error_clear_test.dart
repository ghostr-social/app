import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_screen.dart';

void main() {
  testWidgets('an unreadable ledger can only recover through explicit clear', (
    tester,
  ) async {
    final repository = _CorruptHistoryRepository();
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => WatchHistoryCubit(repository)..load(),
          child: const WatchHistoryScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Watch history unavailable'), findsOneWidget);
    expect(find.byTooltip('Clear watch history'), findsOneWidget);

    await tester.tap(find.byTooltip('Clear watch history'));
    await tester.pumpAndSettle();

    expect(repository.didClear, isTrue);
    expect(find.text('No watched videos yet'), findsOneWidget);
  });
}

class _CorruptHistoryRepository implements WatchHistoryRepository {
  var didClear = false;

  @override
  WatchHistoryRepository snapshotForActiveAccount() => this;

  @override
  Future<List<WatchHistoryEntry>> load() async {
    throw const AppFailure('The ledger cannot be read.');
  }

  @override
  Future<void> record(WatchHistoryEntry entry) async {}

  @override
  Future<void> clear() async => didClear = true;

  @override
  Future<List<VideoPost>> filterUnwatched(List<VideoPost> posts) async => posts;
}
