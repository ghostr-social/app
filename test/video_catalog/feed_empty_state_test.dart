import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('shows the empty feed state', (tester) async {
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [],
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('No videos yet'), findsOneWidget);
  });
}
