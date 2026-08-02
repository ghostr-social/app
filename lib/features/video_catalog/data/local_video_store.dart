import 'dart:convert';

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
    VideoPostStorageMapper mapper = const VideoPostStorageMapper(),
  }) : _mapper = mapper;

  static const _publishedKey = 'ghostr.catalog.published';
  static const _followedKey = 'ghostr.catalog.followed';
  static const _blockedKey = 'ghostr.catalog.blocked';

  final SharedPreferences _preferences;
  final VideoPostStorageMapper _mapper;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() {
    return guardPreferenceStorage(
      'Could not read followed profiles.',
      () => _loadProfileIds(_followedKey),
    );
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    return guardPreferenceStorage(
      'Could not read blocked profiles.',
      () => _loadProfileIds(_blockedKey),
    );
  }

  @override
  Future<List<VideoPost>> loadPublishedPosts() {
    return guardPreferenceStorage(
      'Could not read published videos.',
      _loadPublishedPosts,
    );
  }

  List<VideoPost> _loadPublishedPosts() {
    final raw = _preferences.getString(_publishedKey);
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
    return requirePreferenceWrite(
      'Could not save followed profiles.',
      () => _preferences.setStringList(
        _followedKey,
        _storedProfileIds(profileIds),
      ),
    );
  }

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) {
    return requirePreferenceWrite(
      'Could not save blocked profiles.',
      () => _preferences.setStringList(
        _blockedKey,
        _storedProfileIds(profileIds),
      ),
    );
  }

  @override
  Future<void> savePublishedPosts(List<VideoPost> posts) {
    return guardPreferenceStorage(
      'Could not save published videos.',
      () => _savePublishedPosts(posts),
    );
  }

  Future<void> _savePublishedPosts(List<VideoPost> posts) {
    final payload = posts.map(_mapper.toMap).toList();
    return requirePreferenceWrite(
      'Could not save published videos.',
      () => _preferences.setString(_publishedKey, jsonEncode(payload)),
    );
  }

  Set<ProfileId> _loadProfileIds(String key) {
    return _preferences.getStringList(key)?.map(ProfileId.parse).toSet() ??
        <ProfileId>{};
  }

  List<String> _storedProfileIds(Set<ProfileId> profileIds) {
    return profileIds.map((id) => id.value).toList()..sort();
  }
}
