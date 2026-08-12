import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/presentation/profile_metadata_form_screen.dart';

void main() {
  testWidgets('disables profile controls while a submit is pending', (
    tester,
  ) async {
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
            onSelectPicture: () {},
            onSubmit: (_) {},
          ),
          viewState: const ProfileMetadataFormViewState(isSubmitting: true),
        ),
      ),
    );

    for (final field in tester.widgetList<TextField>(find.byType(TextField))) {
      expect(field.enabled, isFalse);
    }
    expect(find.text('Saving…'), findsOneWidget);
    expect(
      tester.widget<ElevatedButton>(find.byType(ElevatedButton)).onPressed,
      isNull,
    );
    expect(
      tester.widget<OutlinedButton>(find.byType(OutlinedButton)).onPressed,
      isNull,
    );
  });
}
