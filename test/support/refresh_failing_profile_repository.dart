import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

import 'fake_video_catalog_repository.dart';

class RefreshFailingProfileRepository extends FakeVideoCatalogRepository {
  RefreshFailingProfileRepository(this.details)
      : super(forYouFeed: details.posts.toList());

  final ProfileDetails details;
  int loadCount = 0;

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) async {
    loadCount += 1;
    if (loadCount > 1) throw const AppFailure('Profile refresh failed.');
    return details;
  }
}
