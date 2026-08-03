import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('re-tapping Home while on Home triggers a fresh full load',
      (tester) async {
    final catalog = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    await tester.pumpWidget(buildTestApp(buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: catalog,
    )));
    await tester.pumpAndSettle();
    expect(catalog.loadFeedExclusions, [true]);

    await tester.tap(find.text('Home'));
    await tester.pumpAndSettle();

    expect(catalog.loadFeedExclusions, [true, true]);
  });
}
