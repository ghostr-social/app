import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/existing_key_screen.dart';

void main() {
  testWidgets('keeps an existing nsec private while it is entered', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(home: ExistingKeyScreen(onSubmit: (_) {})),
    );

    final field = tester.widget<TextField>(
      find.byKey(const Key('existing-key-nsec-field')),
    );

    expect(field.decoration?.labelText, 'Nostr secret key');
    expect(field.obscureText, isTrue);
    expect(field.enableSuggestions, isFalse);
    expect(field.autocorrect, isFalse);
    expect(field.keyboardType, TextInputType.visiblePassword);
  });
}
