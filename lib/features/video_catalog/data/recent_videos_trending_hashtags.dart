import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';

/// Computes trending hashtags from the newest relay videos as background
/// work, and reuses the answer for a while — discovery garnish must never
/// compete with feeds for bandwidth.
class RecentVideosTrendingHashtags implements TrendingHashtagsSource {
  RecentVideosTrendingHashtags(
    this._source,
    this._scheduler, {
    Clock clock = systemClock,
  }) : _clock = clock;

  static const _timeToLive = Duration(minutes: 15);

  final RemoteVideoSource _source;
  final RetrievalScheduler _scheduler;
  final Clock _clock;
  List<String>? _cached;
  DateTime? _cachedAt;

  @override
  Future<List<String>> trendingHashtags() async {
    final cached = _cached;
    final cachedAt = _cachedAt;
    if (cached != null &&
        cachedAt != null &&
        _clock().difference(cachedAt) < _timeToLive) {
      return cached;
    }
    final posts = await _scheduler.run(
      const RetrievalRequest(
        context: 'discover',
        priority: RetrievalPriority.background,
      ),
      () => _source.loadRemoteFeed(),
    );
    _cached = rankTrendingHashtags(posts);
    _cachedAt = _clock();
    return _cached!;
  }
}
