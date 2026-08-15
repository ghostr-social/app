import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('returning to Home from Search keeps watch exclusions', (
    tester,
  ) async {
    final catalog = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
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
    await tester.tap(find.text('Home'));
    await tester.pumpAndSettle();

    expect(catalog.loadFeedExclusions, [true, true]);
  });
}
