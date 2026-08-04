import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// How many entries each report keeps: enough to recognise a pattern,
/// short enough to stay one readable log line.
const _reportLimit = 5;

/// What one pipeline served that the other did not (plan §5 step 6).
///
/// Membership and order are reported apart: ids only one side served
/// are listed once as [missing] or [extra], and [orderMismatches]
/// compares the ids both sides served, so a single absent row cannot
/// masquerade as a wholesale reordering.
final class FeedParityDivergence {
  const FeedParityDivergence({
    required this.missing,
    required this.extra,
    required this.orderMismatches,
  });

  /// Ids the truth pipeline served and the shadow did not.
  final List<String> missing;

  /// Ids only the shadow pipeline served.
  final List<String> extra;

  /// `index:truth!=shadow` for every shared id the two pipelines
  /// ranked differently.
  final List<String> orderMismatches;

  /// The divergence between [primary] (the truth) and [shadow], or
  /// null when the two pipelines agree.
  static FeedParityDivergence? between(
    List<VideoPost> primary,
    List<VideoPost> shadow,
  ) {
    final primaryIds = _ids(primary);
    final shadowIds = _ids(shadow);
    final shared = primaryIds.toSet().intersection(shadowIds.toSet());
    final divergence = FeedParityDivergence(
      missing: _capped(primaryIds.where((id) => !shared.contains(id))),
      extra: _capped(shadowIds.where((id) => !shared.contains(id))),
      orderMismatches: _capped(_mismatches(
        primaryIds.where(shared.contains).toList(),
        shadowIds.where(shared.contains).toList(),
      )),
    );
    return divergence.isEmpty ? null : divergence;
  }

  bool get isEmpty =>
      missing.isEmpty && extra.isEmpty && orderMismatches.isEmpty;

  @override
  String toString() {
    return 'missing=$missing extra=$extra order=$orderMismatches';
  }

  static List<String> _ids(List<VideoPost> posts) {
    return posts.map((post) => post.id.value).toList(growable: false);
  }

  static Iterable<String> _mismatches(
    List<String> primary,
    List<String> shadow,
  ) sync* {
    final ranked =
        primary.length < shadow.length ? primary.length : shadow.length;
    for (var index = 0; index < ranked; index += 1) {
      if (primary[index] == shadow[index]) continue;
      yield '$index:${primary[index]}!=${shadow[index]}';
    }
  }

  static List<String> _capped(Iterable<String> entries) {
    return entries.take(_reportLimit).toList(growable: false);
  }
}
