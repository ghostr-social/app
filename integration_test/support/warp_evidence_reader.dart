import 'package:ghostr/src/rust/api/warp_evidence_control.dart';

import 'warp_evidence_models.dart';

final class WarpEvidenceReader {
  const WarpEvidenceReader();

  Future<WarpEvidencePage> page({int afterRevision = 0, int limit = 64}) async {
    final encoded = await ffiWarpEvidencePageJson(
      afterPlanRevision: BigInt.from(afterRevision),
      limit: limit,
    );
    return WarpEvidencePage.parse(encoded);
  }

  Future<WarpDecisionEvidence> decisions() async {
    return WarpDecisionEvidence.parse(await ffiWarpDecisionHistoryJson());
  }
}
