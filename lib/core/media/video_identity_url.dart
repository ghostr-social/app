/// Stable URL identity for a video whose delivery signature can be refreshed.
///
/// Query parameters and fragments are delivery metadata, not file identity.
String canonicalVideoIdentityUrl(String raw) {
  final value = raw.trim();
  var end = value.length;
  final query = value.indexOf('?');
  final fragment = value.indexOf('#');
  if (query >= 0 && query < end) end = query;
  if (fragment >= 0 && fragment < end) end = fragment;
  return value.substring(0, end);
}
