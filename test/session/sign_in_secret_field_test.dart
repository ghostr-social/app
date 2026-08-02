import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/sign_in_screen.dart';

void main() {
  testWidgets('protects the nsec field from display and suggestions',
      (tester) async {
    await tester.pumpWidget(MaterialApp(
      home: SignInScreen(onSubmit: (_) {}),
    ));

    final field = tester.widget<TextField>(find.byType(TextField));
    expect(field.obscureText, isTrue);
    expect(field.enableSuggestions, isFalse);
    expect(field.keyboardType, TextInputType.visiblePassword);
  });
}
