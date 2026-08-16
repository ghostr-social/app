import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('re-tapping Home starts a feed with only unwatched videos', (
    tester,
  ) async {
    final posts = [
      samplePost(id: 'first', caption: 'First watched'),
      samplePost(id: 'second', caption: 'Second watched'),
      samplePost(id: 'third', caption: 'Third fresh'),
      samplePost(id: 'fourth', caption: 'Fourth fresh'),
    ];
    final history = FakeWatchHistoryRepository();
    final catalog = FakeVideoCatalogRepository(forYouFeed: posts);
    await tester.pumpWidget(
      buildTestApp(
        buildFakeDependencies(
          session: sampleSession(),
          catalogRepository: catalog,
          overrides: FakeDependencyOverrides(
            watchHistory: history,
            feed: WatchAwareVideoFeedRepository(
              feed: catalog,
              history: history,
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();
    cubit.pageChanged(1);
    await tester.pump();
    await tester.pump();

    await tester.tap(find.text('Home'));
    await tester.pumpAndSettle();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['third', 'fourth']);
    expect(loaded.activeIndex, 0);
    expect(find.text('First watched'), findsNothing);
    expect(find.text('Second watched'), findsNothing);
    expect(find.text('Third fresh').hitTestable(), findsOneWidget);
    expect(find.text('Fourth fresh').hitTestable(), findsNothing);
  });
}
