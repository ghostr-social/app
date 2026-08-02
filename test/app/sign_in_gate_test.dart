import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('shows sign in when no session is restored', (tester) async {
    final dependencies = buildFakeDependencies(
      session: null,
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [samplePost()],
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('Import your Nostr key'), findsOneWidget);
    expect(find.text('For You'), findsNothing);
  });
}
