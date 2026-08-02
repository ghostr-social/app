import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ndk/ndk.dart';

class NdkNostrVideoEventQuery implements NostrVideoEventQueryPort {
  NdkNostrVideoEventQuery(this._ndk);

  static const _videoKinds = [21, 22, 34235, 34236];

  final Ndk _ndk;

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
  }) async {
    try {
      final response = _ndk.requests.query(
        name: 'ghostr-video-feed',
        timeout: const Duration(seconds: 5),
        filter: Filter(
          authors: authorPublicKeys?.map((key) => key.value).toList(),
          kinds: _videoKinds,
          search: searchQuery,
          limit: 80,
        ),
      );
      final events = await response.future;
      events.sort((left, right) => right.createdAt.compareTo(left.createdAt));
      return events;
    } on Object catch (error, stackTrace) {
      throw _failure('Could not load Nostr videos.', error, stackTrace);
    }
  }

  @override
  Future<Metadata?> loadMetadata(NostrPublicKeyHex publicKey) async {
    try {
      return await _ndk.metadata.loadMetadata(publicKey.value);
    } on Object catch (error, stackTrace) {
      throw _failure(
          'Could not load Nostr profile metadata.', error, stackTrace);
    }
  }

  AppFailure _failure(String message, Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'ghostr.nostr.video-query',
      message: message,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
