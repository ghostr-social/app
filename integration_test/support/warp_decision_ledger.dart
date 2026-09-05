import 'warp_evidence_models.dart';

/// Retains every decision record a journey has sampled.
///
/// The engine keeps a bounded decision history, so a record can be evicted
/// before a later stage searches for it. A record's outcome resolves after it
/// is first observed, so the newest copy of a sequence replaces the older one.
final class WarpDecisionLedger {
  final _records = <int, WarpDecisionRecord>{};

  List<WarpDecisionRecord> get records {
    final sorted = _records.values.toList(growable: false)
      ..sort((a, b) => a.sequence.compareTo(b.sequence));
    return sorted;
  }

  void absorb(Iterable<WarpDecisionRecord> sampled) {
    for (final record in sampled) {
      _records[record.sequence] = record;
    }
  }
}
