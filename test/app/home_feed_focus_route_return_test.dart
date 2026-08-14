import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';

import '../support/fakes.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('returning from a routed feed republishes the home focus', (
    tester,
  ) async {
    final home = [samplePost(id: 'home-1'), samplePost(id: 'home-2')];
    final routed = [samplePost(id: 'route-1'), samplePost(id: 'route-2')];
    final focus = FakeFeedFocusPort();
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: home,
      feed: FakeFeedScenario(searchResults: routed),
    );
    await tester.pumpWidget(
      buildTestApp(
        buildFakeDependencies(
          session: sampleSession(),
          catalogRepository: catalog,
        ),
        feedFocus: focus,
      ),
    );
    await tester.pumpAndSettle();
    final homeElement = tester.element(find.byType(FeedScreen));
    homeElement.read<FeedCubit>().pageChanged(1);
    tester
        .widget<FeedScreen>(find.byType(FeedScreen))
        .bindings
        .onOpenHashtag('#route');
    await tester.pumpAndSettle();

    expect(focus.focuses.last.current.id.value, 'route-1');
    await tester.pageBack();
    await tester.pumpAndSettle();

    expect(focus.focuses.last.current.id.value, 'home-2');
    expect(focus.focuses.last.window.map((post) => post.id.value), [
      'home-1',
      'home-2',
    ]);
  });
}
