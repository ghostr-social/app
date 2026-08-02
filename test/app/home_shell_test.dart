import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('shows the authenticated shell with feed content',
      (tester) async {
    final creator = sampleCreator();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [samplePost(creator: creator)],
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('For You'), findsOneWidget);
    expect(find.text('Following'), findsOneWidget);
    expect(find.text('Home'), findsOneWidget);
    expect(find.text('Messages'), findsNothing);
    expect(find.text(creator.displayName), findsOneWidget);
  });
}
