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
