final RegExp hashtagPattern = RegExp(r'#([\p{L}\p{N}_]+)', unicode: true);

List<String> extractHashtags(String text) {
  final found = <String>{};
  for (final match in hashtagPattern.allMatches(text)) {
    found.add(match.group(1)!.toLowerCase());
  }
  return List<String>.unmodifiable(found);
}

String? normalizeHashtag(String raw) {
  var value = raw.trim().toLowerCase();
  if (value.startsWith('#')) value = value.substring(1);
  return value.isEmpty ? null : value;
}

/// Relays match tag values exactly, so a hashtag query must ask for every
/// case form publishers commonly write: as-typed, lower, UPPER, and Title.
List<String> hashtagQueryVariants(String raw) {
  var typed = raw.trim();
  if (typed.startsWith('#')) typed = typed.substring(1);
  final tag = normalizeHashtag(typed);
  if (tag == null) return const <String>[];
  return List<String>.unmodifiable(<String>{
    typed,
    tag,
    tag.toUpperCase(),
    tag[0].toUpperCase() + tag.substring(1),
  });
}
