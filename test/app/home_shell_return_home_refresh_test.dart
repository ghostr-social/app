import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('returning to Home reconciles the current feed', (tester) async {
    final initial = samplePost(id: 'initial');
    final fresh = samplePost(id: 'fresh');
    final catalog = FakeVideoCatalogRepository(forYouFeed: [initial]);
    await tester.pumpWidget(
      buildTestApp(
        buildFakeDependencies(
          session: sampleSession(),
          catalogRepository: catalog,
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    catalog.forYouFeed.add(fresh);
    await tester.tap(find.text('Home'));
    await tester.pumpAndSettle();

    expect(catalog.loadFeedExclusions, [true, false]);
    final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();
    expect((cubit.state as FeedLoaded).posts.map((post) => post.id.value), [
      'initial',
      'fresh',
    ]);
  });
}
