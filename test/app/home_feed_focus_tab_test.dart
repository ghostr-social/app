import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';

import '../support/fakes.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets(
    'returning to Home focuses only a video not watched before exit',
    (tester) async {
      final posts = [
        samplePost(id: 'first'),
        samplePost(id: 'second'),
        samplePost(id: 'third'),
      ];
      final focus = FakeFeedFocusPort();
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
          feedFocus: focus,
        ),
      );
      await tester.pumpAndSettle();
      final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();
      cubit.pageChanged(1);
      await tester.pump();
      await tester.pump();
      final visibleWriteCount = focus.focuses.length;

      await tester.tap(find.text('Search'));
      await tester.pumpAndSettle();
      expect(focus.focuses, hasLength(visibleWriteCount));
      final hiddenWriteCount = focus.focuses.length;
      expect(hiddenWriteCount, visibleWriteCount);
      await tester.tap(find.text('Home'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 350));
      await tester.pump();

      expect(focus.focuses.length, greaterThan(hiddenWriteCount));
      expect(focus.focuses.last.currentIndex, 0);
      expect(focus.focuses.last.current.id.value, 'third');
      expect(focus.focuses.last.window, hasLength(1));
      expect(focus.focuses.last.watched, Duration.zero);
    },
  );
}
