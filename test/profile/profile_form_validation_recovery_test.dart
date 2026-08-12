import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/presentation/profile_metadata_form_screen.dart';

void main() {
  testWidgets('reports invalid picture URLs and permits a corrected submit', (
    tester,
  ) async {
    ProfileMetadata? submitted;
    await tester.pumpWidget(
      MaterialApp(
        home: ProfileMetadataFormScreen(
          configuration: const ProfileMetadataFormConfiguration(
            initial: ProfileFormInitial(
              displayName: 'Nora',
              handle: 'nora',
              pictureUrl: '',
            ),
            title: 'Edit profile',
            submitLabel: 'Save profile',
          ),
          actions: ProfileMetadataFormActions(
            onSubmit: (metadata) => submitted = metadata,
          ),
        ),
      ),
    );
    final picture = find.byKey(const Key('profile-picture-url-field'));

    await tester.enterText(picture, 'avatar.png');
    await tester.tap(find.byKey(const Key('profile-form-submit')));
    await tester.pump();
    expect(find.text('Picture must be an HTTP(S) URL.'), findsOneWidget);
    expect(submitted, isNull);

    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Nora Relay',
    );
    await tester.pump();
    expect(find.text('Picture must be an HTTP(S) URL.'), findsNothing);

    await tester.enterText(picture, 'https://cdn.example/nora.png');
    await tester.tap(find.byKey(const Key('profile-form-submit')));
    expect(submitted?.pictureUrl?.value, 'https://cdn.example/nora.png');
  });
}
