import 'package:ghostr/features/video_catalog/domain/discovery_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import 'discovery_search_fakes.dart';
import 'recording_failure_reporter.dart';

export 'discovery_search_fakes.dart';

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
