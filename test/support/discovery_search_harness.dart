import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/discovery_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import 'recording_failure_reporter.dart';

/// Wires a [DiscoveryVideoSearchRepository] to recording fakes.
class DiscoverySearchHarness {
  DiscoverySearchHarness({
    List<VideoPost> posts = const <VideoPost>[],
    List<ProfileSummary> creators = const <ProfileSummary>[],
  })  : source = RecordingRemoteVideoSource(posts),
        creators = RecordingCreatorSearchSource(creators);

  final RecordingRemoteVideoSource source;
  final RecordingCreatorSearchSource creators;
  final FakeSocialGraph social = FakeSocialGraph();
  final RecordingFailureReporter reporter = RecordingFailureReporter();

  late final repository = DiscoveryVideoSearchRepository(
    videos: source,
    creators: creators,
    social: social,
    failureReporter: reporter,
  );
}

class RecordingRemoteVideoSource implements RemoteVideoSource {
  RecordingRemoteVideoSource(this.posts);

  final List<VideoPost> posts;
  final List<String?> searchQueries = <String?>[];
  final List<Set<String>?> hashtags = <Set<String>?>[];
  final List<DateTime?> olderThans = <DateTime?>[];

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    searchQueries.add(searchQuery);
    this.hashtags.add(hashtags);
    olderThans.add(olderThan);
    return posts;
  }
}

class RecordingCreatorSearchSource implements CreatorSearchSource {
  RecordingCreatorSearchSource(this.creators);

  final List<ProfileSummary> creators;
  final List<String> queries = <String>[];
  AppFailure? failure;

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    queries.add(query);
    if (failure case final AppFailure error) throw error;
    return creators;
  }
}

class FakeSocialGraph implements SocialGraphRepository {
  final Set<ProfileId> blocked = <ProfileId>{};
  final Set<ProfileId> followed = <ProfileId>{};

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => blocked;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => followed;

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => blocked.add(profileId);

  @override
  Future<bool> toggleFollow(ProfileId profileId) async =>
      followed.add(profileId);
}
