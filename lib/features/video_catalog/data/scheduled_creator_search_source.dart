import 'package:ghostr/core/work/retrieval_scheduler.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

/// Runs creator search inside the shared retrieval queue, in the same
/// context as the video search it accompanies.
final class ScheduledCreatorSearchSource implements CreatorSearchSource {
  const ScheduledCreatorSearchSource({
    required CreatorSearchSource source,
    required RetrievalScheduler scheduler,
  })  : _source = source,
        _scheduler = scheduler;

  final CreatorSearchSource _source;
  final RetrievalScheduler _scheduler;

  @override
  Future<List<ProfileSummary>> searchCreators(String query) {
    return _scheduler.run(
      RetrievalRequest(context: 'search:${query.trim().toLowerCase()}'),
      () => _source.searchCreators(query),
    );
  }
}
