import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fake_profile_image_services.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('signed-in user selects and uploads a new profile picture', (
    tester,
  ) async {
    final session = sampleSession();
    final profiles = FakeProfileMetadataRepository();
    final picker = FakeProfileImagePicker()..result = sampleProfileImage();
    final uploader = FakeProfileImageUploader();
    final details = ProfileDetails(
      profile: session.profile,
      posts: const [],
      statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
      relationship: ProfileRelationship(
        isFollowing: false,
        isBlocked: false,
        isCurrentUser: true,
      ),
    );
    final dependencies = buildFakeDependencies(
      session: session,
      profileMetadataRepository: profiles,
      device: FakeDeviceDependencies(
        profileImages: fakeProfileImages(picker: picker, uploader: uploader),
      ),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: const [],
        feed: FakeFeedScenario(profiles: {session.profile.id: details}),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.person_rounded));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Edit profile'));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('profile-picture-picker')));
    await tester.pumpAndSettle();
    expect(find.text('Selected: nora-avatar.png'), findsOneWidget);
    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Nora Relay',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@nora',
    );
    await tester.pump();
    await tester.tap(find.text('Save profile'));
    await tester.pumpAndSettle();

    expect(uploader.uploaded, same(picker.result));
    expect(profiles.savedMetadata?.pictureUrl?.value, uploader.url);
  });
}
