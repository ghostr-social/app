import 'dart:convert';

final class LiveVideoCorpus {
  LiveVideoCorpus._(this._events, this._urls);

  factory LiveVideoCorpus.fromJson(String encoded) {
    final decoded = jsonDecode(encoded);
    if (decoded is! Map<String, dynamic>) {
      throw ArgumentError('The previous corpus must be an object.');
    }
    return LiveVideoCorpus._(
      _strings(decoded['eventIds']),
      _strings(decoded['urls']),
    );
  }

  final Set<String> _events;
  final Set<String> _urls;
  final Map<String, int> _hosts = {};
  int accepted = 0;

  Map<String, int> get hosts => Map.unmodifiable(_hosts);

  bool admit(String eventId, String? url) {
    if (_events.contains(eventId) || (url != null && _urls.contains(url))) {
      return false;
    }
    final host = Uri.tryParse(url ?? '')?.host ?? '';
    final count = _hosts[host] ?? 0;
    if (count >= 5) return false;
    _hosts[host] = count + 1;
    _events.add(eventId);
    if (url != null) _urls.add(url);
    accepted++;
    return true;
  }
}

Set<String> _strings(Object? value) {
  if (value == null) return {};
  if (value is! List || value.any((item) => item is! String)) {
    throw ArgumentError('Corpus entries must be strings.');
  }
  return value.cast<String>().toSet();
}
