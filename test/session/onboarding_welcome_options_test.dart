import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/onboarding_welcome_screen.dart';

void main() {
  testWidgets('offers create-account and existing-key paths', (tester) async {
    var createCount = 0;
    var existingKeyCount = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: OnboardingWelcomeScreen(
          onCreateAccount: () => createCount += 1,
          onUseExistingKey: () => existingKeyCount += 1,
        ),
      ),
    );

    expect(find.bySemanticsLabel('Create a Nostr account'), findsOneWidget);
    expect(find.bySemanticsLabel('Use an existing key'), findsOneWidget);

    await tester.tap(find.text('Create a Nostr account'));
    await tester.tap(find.text('Use an existing key'));

    expect(createCount, 1);
    expect(existingKeyCount, 1);
  });
}
