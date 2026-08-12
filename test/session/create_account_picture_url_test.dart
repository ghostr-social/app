import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/presentation/create_account_profile_screen.dart';

void main() {
  testWidgets('submits the optional profile picture URL', (tester) async {
    ProfileMetadata? submitted;
    await tester.pumpWidget(
      MaterialApp(
        home: CreateAccountProfileScreen(
          initial: null,
          onSubmit: (metadata) => submitted = metadata,
        ),
      ),
    );

    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Nora Relay',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@nora',
    );
    final pictureField = find.byKey(const Key('profile-picture-url-field'));
    expect(
      tester.widget<TextField>(pictureField).decoration?.labelText,
      'Picture URL (optional)',
    );
    await tester.enterText(pictureField, 'https://cdn.example/nora.png');
    await tester.pump();
    await tester.tap(find.byKey(const Key('create-account-submit')));

    expect(submitted?.displayName.value, 'Nora Relay');
    expect(submitted?.handle.value, 'nora');
    expect(submitted?.pictureUrl?.value, 'https://cdn.example/nora.png');
  });
}
