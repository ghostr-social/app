import 'package:ghostr/core/nostr/nostr_event_identity.dart';

/// Bounded causal evidence for accepted Nostr events and deletions.
final class AcceptedNostrEventJournal<K extends Object> {
  static const int maximumEntries = 2048;
  static const int maximumEvidence = 8192;

  final Map<K, _AcceptedEvents> _entries = <K, _AcceptedEvents>{};
  int _evidenceCount = 0;

  Set<NostrEventId> overlay(K key, Set<NostrEventId> relayIds) {
    final accepted = _entries[key];
    return accepted?.overlay(relayIds) ?? Set<NostrEventId>.of(relayIds);
  }

  void recordEvent(K key, NostrEventId eventId) {
    _write(key, (entry) => entry.recordEvent(eventId));
  }

  void recordDeletion(K key, Iterable<NostrEventId> targetIds) {
    final ids = targetIds.toList(growable: false);
    if (ids.isEmpty) return;
    _write(key, (entry) => entry.recordDeletion(ids));
  }

  Set<NostrEventId> pendingTargetIds(K key) {
    return Set<NostrEventId>.of(_entries[key]?.pendingIds ?? const {});
  }

  bool isConfirmedDeleted(K key, NostrEventId eventId) {
    return _entries[key]?.isConfirmedDeleted(eventId) ?? false;
  }

  bool hasEvidence(K key) => _entries.containsKey(key);

  void reconcile(K key, Set<NostrEventId> deletedIds) {
    final entry = _touch(key);
    if (entry == null) return;
    entry.reconcile(deletedIds);
    _removeIfEmpty(key);
  }

  void _write(K key, void Function(_AcceptedEvents) action) {
    final entry = _entry(key);
    final before = entry.length;
    action(entry);
    _evidenceCount += entry.length - before;
    _trimEvidence();
    _removeIfEmpty(key);
  }

  _AcceptedEvents _entry(K key) {
    final existing = _touch(key);
    if (existing != null) return existing;
    if (_entries.length >= maximumEntries) {
      _removeEntry(_entries.keys.first);
    }
    final created = _AcceptedEvents();
    _entries[key] = created;
    return created;
  }

  _AcceptedEvents? _touch(K key) {
    final existing = _entries.remove(key);
    if (existing != null) _entries[key] = existing;
    return existing;
  }

  void _trimEvidence() {
    while (_evidenceCount > maximumEvidence) {
      final key = _entries.keys.first;
      final entry = _entries[key]!;
      final excess = _evidenceCount - maximumEvidence;
      _evidenceCount -= entry.removeOldest(excess);
      _removeIfEmpty(key);
    }
  }

  void _removeIfEmpty(K key) {
    if (_entries[key]?.isEmpty ?? false) _removeEntry(key);
  }

  void _removeEntry(K key) {
    final removed = _entries.remove(key)!;
    _evidenceCount -= removed.length;
  }
}

enum _EvidenceState { active, pendingDeletion, confirmedDeleted }

final class _AcceptedEvents {
  final Map<NostrEventId, _EvidenceState> _evidence = {};

  int get length => _evidence.length;
  bool get isEmpty => _evidence.isEmpty;

  Set<NostrEventId> get pendingIds => <NostrEventId>{
    for (final MapEntry(:key, :value) in _evidence.entries)
      if (value != _EvidenceState.confirmedDeleted) key,
  };

  bool isConfirmedDeleted(NostrEventId id) {
    return _evidence[id] == _EvidenceState.confirmedDeleted;
  }

  Set<NostrEventId> overlay(Set<NostrEventId> relayIds) {
    final result = Set<NostrEventId>.of(relayIds);
    for (final MapEntry(:key, :value) in _evidence.entries) {
      if (value == _EvidenceState.active) {
        result.add(key);
      } else {
        result.remove(key);
      }
    }
    return result;
  }

  void recordEvent(NostrEventId id) {
    _evidence.putIfAbsent(id, () => _EvidenceState.active);
  }

  void recordDeletion(Iterable<NostrEventId> targetIds) {
    for (final id in targetIds) {
      if (isConfirmedDeleted(id)) continue;
      _replace(id, _EvidenceState.pendingDeletion);
    }
  }

  void reconcile(Set<NostrEventId> deletedIds) {
    for (final id in deletedIds) {
      if (_evidence.containsKey(id)) {
        _replace(id, _EvidenceState.confirmedDeleted);
      }
    }
  }

  int removeOldest(int count) {
    var removed = 0;
    while (removed < count && _evidence.isNotEmpty) {
      _evidence.remove(_evidence.keys.first);
      removed += 1;
    }
    return removed;
  }

  void _replace(NostrEventId id, _EvidenceState state) {
    _evidence.remove(id);
    _evidence[id] = state;
  }
}
