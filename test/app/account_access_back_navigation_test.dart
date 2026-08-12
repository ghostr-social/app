import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('account access branches offer an accessible return to welcome', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Use an existing key'));
    await tester.pumpAndSettle();
    await _returnToWelcome(tester);

    await tester.tap(find.text('Create a Nostr account'));
    await tester.pumpAndSettle();
    await _returnToWelcome(tester);
    semantics.dispose();
  });
}

Future<void> _returnToWelcome(WidgetTester tester) async {
  final back = find.byTooltip('Back');
  expect(back, findsOneWidget);
  expect(tester.getSemantics(back).tooltip, 'Back');
  await tester.tap(back);
  await tester.pumpAndSettle();
  expect(find.text('Welcome to Ghostr'), findsOneWidget);
  expect(find.text('Create a Nostr account'), findsOneWidget);
  expect(find.text('Use an existing key'), findsOneWidget);
}
