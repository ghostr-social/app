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
  testWidgets('retries a failed watch history load successfully', (
    tester,
  ) async {
    final repository = _FailingOnceWatchHistoryRepository();

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
    expect(find.text('History store offline.'), findsOneWidget);

    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.text('Watch history unavailable'), findsNothing);
    expect(find.text('A relay-side banger'), findsOneWidget);
  });
}

class _FailingOnceWatchHistoryRepository implements WatchHistoryRepository {
  var _failNextLoad = true;

  @override
  WatchHistoryRepository snapshotForActiveAccount() => this;

  @override
  Future<List<WatchHistoryEntry>> load() async {
    if (_failNextLoad) {
      _failNextLoad = false;
      throw const AppFailure('History store offline.');
    }
    return [
      WatchHistoryEntry(
        videoId: 'e:video-1',
        title: 'A relay-side banger',
        creatorName: 'Nora Relay',
        watchedAt: DateTime(2026, 3, 12, 10, 30),
      ),
    ];
  }

  @override
  Future<void> record(WatchHistoryEntry entry) async {}

  @override
  Future<void> clear() async {}

  @override
  Future<List<VideoPost>> filterUnwatched(List<VideoPost> posts) async => posts;
}
