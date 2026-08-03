import 'dart:convert';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:shared_preferences/shared_preferences.dart';

class LocalVideoStore implements SocialGraphStore, PublishedVideoStore {
  LocalVideoStore(
    this._preferences, {
    required AccountStorageScope accountScope,
    VideoPostStorageMapper mapper = const VideoPostStorageMapper(),
  })  : _accountScope = accountScope,
        _mapper = mapper,
        _pinnedAccount = null;

  const LocalVideoStore._(
    this._preferences,
    this._accountScope,
    this._mapper,
    this._pinnedAccount,
  );

  static const _publishedKey = 'ghostr.catalog.published';
  static const _followedKey = 'ghostr.catalog.followed';
  static const _blockedKey = 'ghostr.catalog.blocked';

  final SharedPreferences _preferences;
  final AccountStorageScope _accountScope;
  final VideoPostStorageMapper _mapper;
  final AccountStorageKey? _pinnedAccount;

  @override
  LocalVideoStore snapshotForActiveAccount() {
    if (_pinnedAccount != null) return this;
    return LocalVideoStore._(
      _preferences,
      _accountScope,
      _mapper,
      _accountScope.capture(),
    );
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() {
    final key = _account.key(_followedKey);
    return guardPreferenceStorage(
      'Could not read followed profiles.',
      () => _loadProfileIds(key),
    );
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    final key = _account.key(_blockedKey);
    return guardPreferenceStorage(
      'Could not read blocked profiles.',
      () => _loadProfileIds(key),
    );
  }

  @override
  Future<List<VideoPost>> loadPublishedPosts() {
    final key = _account.key(_publishedKey);
    return guardPreferenceStorage(
      'Could not read published videos.',
      () => _loadPublishedPosts(key),
    );
  }

  List<VideoPost> _loadPublishedPosts(String key) {
    final raw = _preferences.getString(key);
    if (raw == null || raw.isEmpty) {
      return const <VideoPost>[];
    }
    final decoded = jsonDecode(raw) as List<dynamic>;
    return decoded
        .map((entry) => _mapper.fromMap(entry as Map<String, dynamic>))
        .toList()
      ..sort((left, right) => right.publishedAt.compareTo(left.publishedAt));
  }

  @override
  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds) {
    final key = _account.key(_followedKey);
    return requirePreferenceWrite(
      'Could not save followed profiles.',
      () => _preferences.setStringList(
        key,
        _storedProfileIds(profileIds),
      ),
    );
  }

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) {
    final key = _account.key(_blockedKey);
    return requirePreferenceWrite(
      'Could not save blocked profiles.',
      () => _preferences.setStringList(
        key,
        _storedProfileIds(profileIds),
      ),
    );
  }

  @override
  Future<void> savePublishedPosts(List<VideoPost> posts) {
    final key = _account.key(_publishedKey);
    return guardPreferenceStorage(
      'Could not save published videos.',
      () => _savePublishedPosts(posts, key),
    );
  }

  Future<void> _savePublishedPosts(List<VideoPost> posts, String key) {
    final payload = posts.map(_mapper.toMap).toList();
    return requirePreferenceWrite(
      'Could not save published videos.',
      () => _preferences.setString(
        key,
        jsonEncode(payload),
      ),
    );
  }

  Set<ProfileId> _loadProfileIds(String key) {
    return _preferences.getStringList(key)?.map(ProfileId.parse).toSet() ??
        <ProfileId>{};
  }

  List<String> _storedProfileIds(Set<ProfileId> profileIds) {
    return profileIds.map((id) => id.value).toList()..sort();
  }

  AccountStorageKey get _account => _pinnedAccount ?? _accountScope.capture();

  @override
  NostrPublicKeyHex get accountPublicKey => _account.account;
}
