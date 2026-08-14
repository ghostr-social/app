import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

enum VideoRepostTarget { specificEvent, coordinate }

/// The outer event responsible for an original video's feed occurrence.
final class VideoRepostAttribution {
  const VideoRepostAttribution({
    required this.eventId,
    required this.reposter,
    required this.repostedAt,
    required this.target,
  });

  final NostrEventId eventId;
  final ProfileSummary reposter;
  final DateTime repostedAt;
  final VideoRepostTarget target;
}
