import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/pending_profile_loads.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('relay metadata replaces cached self header before posts load', (
    tester,
  ) async {
    final cached = sampleSession().profile;
    final refreshed = ProfileSummary(
      id: cached.id,
      displayName: 'Relay Nora',
      handle: '@relay_nora',
      avatarUrl: 'https://example.com/nora.jpg',
    );
    final profiles = FakeProfileMetadataRepository()..cached = refreshed;
    final loads = PendingProfileLoads();
    ProfileSummary? sessionUpdate;

    await tester.pumpWidget(
      profileScreenHarness(
        profile: loads,
        viewer: cached,
        profileId: cached.id,
        metadata: profiles,
        onCurrentProfileUpdated: (profile) => sessionUpdate = profile,
      ),
    );
    await tester.pump();

    expect(find.text('Relay Nora'), findsOneWidget);
    expect(find.text('@relay_nora'), findsOneWidget);
    expect(sessionUpdate, same(refreshed));

    loads.pending.complete(ProfileDetails.empty(cached));
    await tester.pump();
    expect(find.text('Relay Nora'), findsOneWidget);
  });
}
