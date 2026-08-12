import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/presentation/profile_metadata_form_screen.dart';

void main() {
  testWidgets('identifies and disables a pending photo picker', (tester) async {
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
          viewState: const ProfileMetadataFormViewState(
            isSelectingPicture: true,
          ),
        ),
      ),
    );

    final picker = tester.widget<OutlinedButton>(
      find.byKey(const Key('profile-picture-picker')),
    );

    expect(find.text('Opening photos…'), findsOneWidget);
    expect(picker.onPressed, isNull);
    expect(
      tester.widget<ElevatedButton>(find.byType(ElevatedButton)).onPressed,
      isNull,
    );
    for (final field in tester.widgetList<TextField>(find.byType(TextField))) {
      expect(field.enabled, isFalse);
    }
  });
}
