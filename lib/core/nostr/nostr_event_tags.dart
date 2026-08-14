part of 'nostr_event_record.dart';

class NostrEventTags extends IterableBase<List<String>> {
  NostrEventTags._(this._values);

  factory NostrEventTags.parse(Iterable<List<String>> rawTags) {
    final tags = rawTags.map(_validatedTag).toList(growable: false);
    return NostrEventTags._(List<List<String>>.unmodifiable(tags));
  }

  final List<List<String>> _values;

  @override
  Iterator<List<String>> get iterator => _values.iterator;

  Iterable<String> values(String name) sync* {
    for (final tag in _values) {
      if (tag.first == name && tag.length > 1) yield tag[1];
    }
  }

  List<List<String>> toRaw() {
    return List<List<String>>.unmodifiable(
      _values.map(List<String>.unmodifiable),
    );
  }
}

List<String> _validatedTag(List<String> rawTag) {
  if (rawTag.isEmpty) throw const FormatException('Nostr tag cannot be empty.');
  final tag = List<String>.of(rawTag);
  tag[0] = _required(tag[0], 'Nostr tag name');
  return List<String>.unmodifiable(tag);
}

String _required(String raw, String label) {
  final value = raw.trim();
  if (value.isEmpty) throw FormatException('$label cannot be empty.');
  return value;
}

String _requiredExact(String raw, String label) {
  if (raw.trim().isEmpty) throw FormatException('$label cannot be empty.');
  return raw;
}
