part of 'live_video_journey.dart';

extension LiveVideoJourneyEvidence on LiveVideoJourney {
  Future<void> captureEvidence() async {
    if (runtime.environment.delivery == null) return;
    try {
      final head = await ffiWarpEvidencePageJson(
        afterPlanRevision: BigInt.zero,
        limit: 1,
      ).timeout(const Duration(seconds: 5));
      final metadata = jsonDecode(head) as Map<String, Object?>;
      final plans = metadata['plan_page']! as Map<String, Object?>;
      final latest = plans['latest_retained_revision']! as int;
      final page = await ffiWarpEvidencePageJson(
        afterPlanRevision: BigInt.from(latest > 8 ? latest - 8 : 0),
        limit: 8,
      ).timeout(const Duration(seconds: 5));
      final decisions = await ffiWarpDecisionHistoryJson().timeout(
        const Duration(seconds: 5),
      );
      evidence.add({
        'elapsedMs': log.watch.elapsedMilliseconds,
        'page': jsonDecode(page),
        'decisions': _recentDecisions(decisions),
      });
    } on Object catch (error) {
      log.add('evidence_error', {'error': '$error'});
    }
  }
}

Map<String, Object?> _recentDecisions(String encoded) {
  final root = jsonDecode(encoded) as Map<String, Object?>;
  final history = root['decisions']! as Map<String, Object?>;
  final records = history['records']! as List<Object?>;
  return {'records': records.reversed.take(2).toList().reversed.toList()};
}
