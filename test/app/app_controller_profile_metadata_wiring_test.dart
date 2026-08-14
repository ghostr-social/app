import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'profile controller refreshes metadata and reports session update',
    () async {
      final viewer = sampleSession().profile;
      final refreshed = ProfileSummary(
        id: viewer.id,
        displayName: 'Relay Nora',
        handle: '@relay_nora',
        avatarUrl: null,
      );
      final metadata = FakeProfileMetadataRepository()..cached = refreshed;
      final factory = AppControllerFactory(
        buildFakeDependencies(
          catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
          overrides: FakeDependencyOverrides(
            profileMetadataRepository: metadata,
          ),
        ),
      );
      ProfileSummary? sessionUpdate;
      final cubit = factory.profile(
        viewer,
        viewer.id,
        onCurrentProfileUpdated: (profile) => sessionUpdate = profile,
      );
      addTearDown(cubit.close);

      await cubit.load();

      expect(cubit.state.details?.profile, same(refreshed));
      expect(sessionUpdate, same(refreshed));
    },
  );
}
