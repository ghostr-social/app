import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';
import 'package:ghostr/features/video_catalog/data/scheduled_creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

import '../support/sample_data.dart';

void main() {
  test('creator search waits its turn in the shared retrieval queue',
      () async {
    final scheduler = RetrievalScheduler(maxConcurrent: 1);
    final inner = _RecordingCreators();
    final source =
        ScheduledCreatorSearchSource(source: inner, scheduler: scheduler);
    final gate = Completer<void>();
    unawaited(scheduler.run(
      const RetrievalRequest(context: 'feed'),
      () => gate.future,
    ));

    final creators = source.searchCreators(' Alice ');
    expect(inner.queries, isEmpty);

    gate.complete();
    expect((await creators).single.displayName, 'Nora Relay');
    expect(inner.queries, [' Alice ']);
  });
}

class _RecordingCreators implements CreatorSearchSource {
  final List<String> queries = <String>[];

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    queries.add(query);
    return [sampleCreator()];
  }
}
