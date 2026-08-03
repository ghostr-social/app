import 'dart:collection';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';

class NostrEventRecord {
  NostrEventRecord({
    required this.identity,
    required List<List<String>> tags,
    required this.content,
    required this.createdAt,
  }) : tags = NostrEventTags.parse(tags) {
    if (createdAt < 0) {
      throw const FormatException('Nostr event timestamp cannot be negative.');
    }
  }

  final NostrEventIdentity identity;
  final NostrEventTags tags;
  final String content;
  final int createdAt;

  NostrEventId get id => identity.id;

  NostrPublicKeyHex get authorPublicKeyHex => identity.authorPublicKeyHex;

  NostrEventKind get kind => identity.kind;

  Iterable<String> tagValues(String name) => tags.values(name);
}

class NostrUnsignedEvent {
  NostrUnsignedEvent({
    required int kind,
    required List<List<String>> tags,
    required this.content,
  })  : kind = NostrEventKind.parse(kind),
        tags = NostrEventTags.parse(tags);

  final NostrEventKind kind;
  final NostrEventTags tags;
  final String content;

  NostrEventRecord toRecord({
    required String id,
    required String authorPublicKeyHex,
    required int createdAt,
  }) {
    return NostrEventRecord(
      identity: NostrEventIdentity.parse(
        id: id,
        authorPublicKeyHex: authorPublicKeyHex,
        kind: kind.value,
      ),
      tags: tags.toRaw(),
      content: content,
      createdAt: createdAt,
    );
  }
}

class NostrEventQuery {
  NostrEventQuery({
    required List<int> kinds,
    NostrEventQueryScope? scope,
    List<NostrTagFilter> tagFilters = const <NostrTagFilter>[],
    this.limit = 500,
    this.until,
    String? search,
  })  : kinds = List<NostrEventKind>.unmodifiable(
          kinds.map(NostrEventKind.parse),
        ),
        scope = scope ?? NostrEventQueryScope(),
        tagFilters = List<NostrTagFilter>.unmodifiable(tagFilters),
        search = _normalizedSearch(search) {
    if (limit <= 0) {
      throw const FormatException('Query limit must be positive.');
    }
    if (until case final int value when value < 0) {
      throw const FormatException('Query until cannot be negative.');
    }
  }

  final List<NostrEventKind> kinds;
  final NostrEventQueryScope scope;
  final List<NostrTagFilter> tagFilters;
  final int limit;
  final int? until;

  /// NIP-50 full-text term. Matching is relay-defined, so [matches] ignores it.
  final String? search;

  static String? _normalizedSearch(String? raw) {
    final value = raw?.trim();
    return value == null || value.isEmpty ? null : value;
  }

  List<NostrPublicKeyHex> get authors => scope.authors;

  List<NostrEventId> get eventTags => scope.eventTags;

  bool matches(NostrEventRecord event) {
    return _matchesKind(event) &&
        _matchesAuthor(event) &&
        _matchesEventTags(event) &&
        _matchesUntil(event) &&
        tagFilters.every((filter) => filter.matches(event));
  }

  bool _matchesUntil(NostrEventRecord event) {
    return until == null || event.createdAt <= until!;
  }

  bool _matchesKind(NostrEventRecord event) => kinds.contains(event.kind);

  bool _matchesAuthor(NostrEventRecord event) {
    return authors.isEmpty || authors.contains(event.authorPublicKeyHex);
  }

  bool _matchesEventTags(NostrEventRecord event) {
    return eventTags.isEmpty || event.tagValues('e').any(eventTags.contains);
  }
}

class NostrEventQueryScope {
  NostrEventQueryScope({
    List<NostrPublicKeyHex> authors = const <NostrPublicKeyHex>[],
    List<NostrEventId> eventTags = const <NostrEventId>[],
  })  : authors = List<NostrPublicKeyHex>.unmodifiable(authors),
        eventTags = List<NostrEventId>.unmodifiable(eventTags);

  factory NostrEventQueryScope.parse({
    List<String> authors = const <String>[],
    List<String> eventTags = const <String>[],
  }) {
    return NostrEventQueryScope(
      authors: authors.map(NostrPublicKeyHex.parse).toList(growable: false),
      eventTags: eventTags.map(NostrEventId.parse).toList(growable: false),
    );
  }

  final List<NostrPublicKeyHex> authors;
  final List<NostrEventId> eventTags;
}

class NostrTagFilter {
  NostrTagFilter({required String name, required List<String> values})
      : name = _required(name, 'Nostr filter tag'),
        values = List<String>.unmodifiable(values.map((value) {
          return _required(value, 'Nostr filter value');
        }));

  final String name;
  final List<String> values;

  bool matches(NostrEventRecord event) {
    return event.tagValues(name).any(values.contains);
  }
}

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
