import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';
import 'package:ghostr/features/video_catalog/data/scheduled_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('loads go through the shared queue and focus their context', () async {
    final scheduler = RetrievalScheduler(maxConcurrent: 1);
    final started = <String>[];
    final inner = _RecordingSource(() => started.add('load'));
    final source =
        ScheduledRemoteVideoSource(source: inner, scheduler: scheduler);
    final gate = Completer<void>();

    unawaited(scheduler.run(
      const RetrievalRequest(context: 'feed'),
      () => gate.future,
    ));
    final load = source.loadRemoteFeed(searchQuery: 'Ghost');
    final other = scheduler.run(
      const RetrievalRequest(context: 'other'),
      () async => started.add('other'),
    );
    final probe = scheduler.run(
      const RetrievalRequest(
        context: 'search:ghost',
        priority: RetrievalPriority.background,
      ),
      () async => started.add('probe'),
    );
    expect(inner.searchQueries, isEmpty);

    gate.complete();
    final loaded = await load;
    await Future.wait([other, probe]);

    expect(loaded, hasLength(1));
    expect(started, ['load', 'probe', 'other']);
    expect(inner.searchQueries, ['Ghost']);
    expect(inner.olderThans, [null]);
  });
}

class _RecordingSource implements RemoteVideoSource {
  _RecordingSource(this.onLoad);

  final void Function() onLoad;
  final List<String?> searchQueries = <String?>[];
  final List<DateTime?> olderThans = <DateTime?>[];

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    onLoad();
    searchQueries.add(searchQuery);
    olderThans.add(olderThan);
    return [samplePost()];
  }
}
