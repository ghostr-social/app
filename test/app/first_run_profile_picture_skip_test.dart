import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_image_services.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('new user can continue without a failed optional picture', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final account = accountCreationAccount();
    final generator = FakeNostrAccountGenerator(account);
    final provisioning = FakeAccountProvisioningRepository();
    final profiles = FakeProfileMetadataRepository();
    final picker = FakeProfileImagePicker()..result = sampleProfileImage();
    final uploader = FakeProfileImageUploader()
      ..failure = const AppFailure('Blossom unavailable.');
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      overrides: FakeDependencyOverrides(
        accountGenerator: generator,
        accountProvisioningRepository: provisioning,
        profileMetadataRepository: profiles,
      ),
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
    await _enterProfile(tester);
    await tester.tap(find.text('Create account'));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('backup-confirmation')));
    await tester.pump();
    await tester.tap(find.text('Finish'));
    await tester.pumpAndSettle();

    expect(find.text('Blossom unavailable.'), findsOneWidget);
    expect(find.text('Continue without selected picture'), findsOneWidget);
    expect(
      find.bySemanticsLabel('Continue without selected picture'),
      findsOneWidget,
    );
    semantics.dispose();
    await tester.tap(find.text('Continue without selected picture'));
    await tester.pump();
    await tester.tap(find.text('Finish'));
    await tester.pumpAndSettle();

    expect(generator.generationCount, 1);
    expect(provisioning.stageCount, 1);
    expect(provisioning.activateCount, 2);
    expect(provisioning.discardCount, 0);
    expect(profiles.savedMetadata?.displayName.value, 'Nora Relay');
    expect(profiles.savedMetadata?.handle.value, 'nora');
    expect(profiles.savedMetadata?.pictureUrl, isNull);
    expect(find.text('Home'), findsOneWidget);
  });
}

Future<void> _enterProfile(WidgetTester tester) async {
  await tester.enterText(
    find.byKey(const Key('profile-display-name-field')),
    'Nora Relay',
  );
  await tester.enterText(
    find.byKey(const Key('profile-handle-field')),
    '@nora',
  );
  await tester.pump();
}
