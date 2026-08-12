import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

/// One entry of the viewer's block list, described for display.
class BlockedAccount {
  BlockedAccount({required this.id, String? displayName})
      : displayName = _cleaned(displayName);

  static const _shortPrefixLength = 11;
  static const _shortSuffixLength = 4;

  final ProfileId id;
  final String? displayName;

  /// What the viewer recognizes the account by.
  String get label => displayName ?? shortId;

  /// The npub shortened to its recognizable edges.
  String get shortId {
    final value = id.value;
    const edges = _shortPrefixLength + _shortSuffixLength + 1;
    if (value.length <= edges) return value;
    final tail = value.substring(value.length - _shortSuffixLength);
    return '${value.substring(0, _shortPrefixLength)}…$tail';
  }

  static String? _cleaned(String? value) {
    final trimmed = value?.trim();
    return trimmed == null || trimmed.isEmpty ? null : trimmed;
  }
}
