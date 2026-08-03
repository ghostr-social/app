import 'dart:developer';

import 'package:ghostr/features/video_catalog/data/creator_profile_summary.dart';
import 'package:ghostr/features/video_catalog/data/nostr_profile_search_port.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

class MetadataCreatorSearchSource implements CreatorSearchSource {
  const MetadataCreatorSearchSource(this._port);

  final NostrProfileSearchPort _port;

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    final metadata = await _port.searchProfiles(query);
    final creators = <ProfileSummary>[];
    for (final entry in metadata) {
      try {
        creators.add(creatorProfileSummary(entry.pubKey, entry));
      } on Object catch (error, stackTrace) {
        log(
          'Skipping a malformed creator search result.',
          name: 'ghostr.video.search',
          error: error,
          stackTrace: stackTrace,
        );
      }
    }
    return creators;
  }
}
