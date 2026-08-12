import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/create_account_profile_screen.dart';

void main() {
  testWidgets('requires a name and handle before account creation', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: CreateAccountProfileScreen(initial: null, onSubmit: (_) {}),
      ),
    );
    final submit = find.byKey(const Key('create-account-submit'));

    expect(tester.widget<ElevatedButton>(submit).onPressed, isNull);
    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Nora Relay',
    );
    await tester.pump();
    expect(tester.widget<ElevatedButton>(submit).onPressed, isNull);

    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@nora',
    );
    await tester.pump();
    expect(tester.widget<ElevatedButton>(submit).onPressed, isNotNull);
  });
}
