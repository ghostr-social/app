import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';

import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows a retryable creator-profile load error', (tester) async {
    final repository = _FailingProfileRepository();
    await tester.pumpWidget(profileScreenHarness(
      profile: repository,
      viewer: sampleSession().profile,
      profileId: sampleCreator().id,
    ));
    await tester.pumpAndSettle();

    expect(find.text('Profile unavailable'), findsOneWidget);
    expect(find.text('Creator lookup failed.'), findsOneWidget);
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();
    expect(repository.loadCount, 2);
  });
}

class _FailingProfileRepository implements VideoProfileRepository {
  int loadCount = 0;

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) async {
    loadCount += 1;
    throw const AppFailure('Creator lookup failed.');
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}
