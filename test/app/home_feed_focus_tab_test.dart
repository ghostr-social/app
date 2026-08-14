import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';

import '../support/fakes.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('tab visibility retains and restores the complete home focus', (
    tester,
  ) async {
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final focus = FakeFeedFocusPort();
    await tester.pumpWidget(
      buildTestApp(
        buildFakeDependencies(
          session: sampleSession(),
          catalogRepository: FakeVideoCatalogRepository(forYouFeed: posts),
        ),
        feedFocus: focus,
      ),
    );
    await tester.pumpAndSettle();
    final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();
    final visibleWriteCount = focus.focuses.length;

    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    expect(focus.focuses, hasLength(visibleWriteCount));
    cubit.pageChanged(1);
    final hiddenWriteCount = focus.focuses.length;
    expect(hiddenWriteCount, visibleWriteCount);
    await tester.tap(find.text('Home'));
    await tester.pumpAndSettle();

    expect(focus.focuses.length, greaterThan(hiddenWriteCount));
    expect(focus.focuses.last.currentIndex, 1);
    expect(focus.focuses.last.window, hasLength(2));
    expect(focus.focuses.last.watched, Duration.zero);
  });
}
