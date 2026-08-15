List<Map<String, Object>> legacyWatchHistoryEntries({
  required int count,
  required DateTime oldest,
}) {
  return List<Map<String, Object>>.generate(count, (index) {
    return <String, Object>{
      'videoId': 'e:legacy-$index',
      'title': 'Legacy watch $index',
      'creatorName': 'Nora Relay',
      'watchedAt': oldest.add(Duration(minutes: index)).toIso8601String(),
    };
  });
}
