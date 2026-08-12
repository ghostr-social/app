import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/startup_gate.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows a retryable startup error then opens the app', (
    tester,
  ) async {
    var attempts = 0;
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
    );
    await tester.pumpWidget(
      StartupGate(
        loadDependencies: () async {
          attempts += 1;
          if (attempts == 1) throw StateError('preferences unavailable');
          return dependencies;
        },
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Ghostr could not start'), findsOneWidget);
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.text('Welcome to Ghostr'), findsOneWidget);
    expect(find.text('Create a Nostr account'), findsOneWidget);
    expect(find.text('Use an existing key'), findsOneWidget);
    expect(attempts, 2);
  });
}
