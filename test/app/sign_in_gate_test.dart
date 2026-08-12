import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('shows account access choices when no session is restored', (
    tester,
  ) async {
    final dependencies = buildFakeDependencies(
      session: null,
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('Welcome to Ghostr'), findsOneWidget);
    expect(find.text('Create a Nostr account'), findsOneWidget);
    expect(find.text('Use an existing key'), findsOneWidget);
    expect(find.byTooltip('Open profile'), findsNothing);
  });
}
