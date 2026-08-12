import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_image_services.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('new user selects and uploads a profile picture after backup', (
    tester,
  ) async {
    final generator = FakeNostrAccountGenerator(accountCreationAccount());
    final profiles = FakeProfileMetadataRepository();
    final picker = FakeProfileImagePicker()..result = sampleProfileImage();
    final uploader = FakeProfileImageUploader();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      accountGenerator: generator,
      profileMetadataRepository: profiles,
      device: FakeDeviceDependencies(
        profileImages: fakeProfileImages(picker: picker, uploader: uploader),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Create a Nostr account'));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('profile-picture-picker')));
    await tester.pumpAndSettle();
    expect(find.text('Selected: nora-avatar.png'), findsOneWidget);
    expect(generator.generationCount, 0);
    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Nora Relay',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@nora',
    );
    await tester.pump();
    final create = find.byKey(const Key('create-account-submit'));
    expect(tester.widget<ElevatedButton>(create).onPressed, isNotNull);
    await tester.tap(find.text('Create account'));
    await tester.pumpAndSettle();
    expect(find.text('Back up your private key'), findsOneWidget);

    await tester.tap(find.byKey(const Key('backup-confirmation')));
    await tester.pump();
    await tester.tap(find.text('Finish'));
    await tester.pumpAndSettle();

    expect(generator.generationCount, 1);
    expect(uploader.uploaded, same(picker.result));
    expect(profiles.savedMetadata?.pictureUrl?.value, uploader.url);
  });
}
