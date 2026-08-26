import 'dart:convert';

part 'warp_evidence_json.dart';
part 'warp_evidence_decisions.dart';
part 'warp_evidence_metrics.dart';
part 'warp_evidence_metrics_adaptation.dart';
part 'warp_evidence_plan.dart';

final class WarpEvidencePage {
  const WarpEvidencePage({required this.planPage, required this.evaluation});

  factory WarpEvidencePage.parse(String encoded) {
    final root = _warpObject(jsonDecode(encoded), r'$');
    _warpSchema(root);
    return WarpEvidencePage(
      planPage: WarpPlanPage.fromJson(_warpChild(root, 'plan_page')),
      evaluation: WarpEvaluationSnapshot.fromJson(
        _warpChild(root, 'evaluation'),
      ),
    );
  }

  final WarpPlanPage planPage;
  final WarpEvaluationSnapshot evaluation;
}

final class WarpPlanPage {
  const WarpPlanPage({
    required this.oldestRetainedRevision,
    required this.latestRetainedRevision,
    required this.cursorTruncated,
    required this.hasMore,
    required this.records,
  });

  factory WarpPlanPage.fromJson(Map<String, Object?> json) => WarpPlanPage(
    oldestRetainedRevision: _warpInt(json, 'oldest_retained_revision'),
    latestRetainedRevision: _warpInt(json, 'latest_retained_revision'),
    cursorTruncated: _warpBool(json, 'cursor_truncated'),
    hasMore: _warpBool(json, 'has_more'),
    records: _warpList(json, 'records')
        .map((item) => WarpPlanEvidence.fromJson(_warpObject(item, 'records')))
        .toList(growable: false),
  );

  final int oldestRetainedRevision;
  final int latestRetainedRevision;
  final bool cursorTruncated;
  final bool hasMore;
  final List<WarpPlanEvidence> records;
}

final class WarpEvaluationSnapshot {
  const WarpEvaluationSnapshot({
    required this.userVisible,
    required this.efficiency,
    required this.budget,
    required this.readiness,
    required this.adaptation,
    required this.semantics,
    required this.integrity,
  });

  factory WarpEvaluationSnapshot.fromJson(Map<String, Object?> json) =>
      WarpEvaluationSnapshot(
        userVisible: _warpUserVisible(_warpChild(json, 'user_visible')),
        efficiency: _warpEfficiency(_warpChild(json, 'efficiency')),
        budget: _warpBudget(_warpChild(json, 'budget')),
        readiness: _warpReadiness(_warpChild(json, 'readiness')),
        adaptation: _warpAdaptation(_warpChild(json, 'adaptation')),
        semantics: _warpSemantics(_warpChild(json, 'semantics')),
        integrity: _warpIntegrity(_warpChild(json, 'integrity')),
      );

  final WarpUserVisibleMetrics userVisible;
  final WarpEfficiencyMetrics efficiency;
  final WarpBudgetMetrics budget;
  final WarpReadinessMetrics readiness;
  final WarpAdaptationMetrics adaptation;
  final WarpSemanticsMetrics semantics;
  final WarpIntegrityMetrics integrity;
}
