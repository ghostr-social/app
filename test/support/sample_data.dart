import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

import 'nostr_test_values.dart';

ProfileSummary sampleCreator({
  String id = 'creator-1',
  String displayName = 'Nora Relay',
  String? avatarUrl,
}) {
  return ProfileSummary(
    id: ProfileId.parse(id),
    displayName: displayName,
    handle: '@${displayName.toLowerCase().replaceAll(' ', '')}',
    avatarUrl: avatarUrl,
  );
}

VideoPost samplePost({
  String id = 'post-1',
  String caption = 'A relay-side banger',
  List<String> hashtags = const <String>[],
  ProfileSummary? creator,
  NostrEventReference? nostrReference,
  DateTime? publishedAt,
}) {
  return VideoPost(
    identity: VideoPostIdentity(
      id: VideoPostId.parse(id),
      creator: creator ?? sampleCreator(),
      nostrReference: nostrReference,
    ),
    content: VideoPostContent(
      caption: caption,
      songName: 'Original sound',
      media: VideoMediaSource.remote('https://example.com/video/$id.mp4'),
      publishedAt: publishedAt ?? DateTime(2026, 3, 12),
      hashtags: hashtags,
    ),
    metrics: VideoPostMetrics(
      likeCount: 42,
      commentCount: 9,
      viewerHasLiked: false,
    ),
  );
}

UserSession sampleSession() {
  return UserSession.fromIdentity(
    AuthSecret.parse(testNsec),
    NostrIdentity.parse(
      publicKeyHex: testViewerPublicKey,
      npub: testViewerNpub,
    ),
  );
}

ProfileDetails sampleProfileDetails(
    {ProfileSummary? profile, List<VideoPost>? posts}) {
  final summary = profile ?? sampleCreator();
  final profilePosts = posts ?? <VideoPost>[samplePost(creator: summary)];
  return ProfileDetails(
    profile: summary,
    posts: profilePosts,
    statistics: ProfileStatistics(totalLikes: 42, followingCount: 3),
    relationship: ProfileRelationship(
      isFollowing: false,
      isBlocked: false,
      isCurrentUser: false,
    ),
  );
}

SelectedMedia sampleMedia() => SelectedMedia(
      path: '/tmp/ghostr-test.mp4',
      source: MediaPickSource.gallery,
      label: 'ghostr-test.mp4',
      mimeType: VideoMimeType.fromFileName('ghostr-test.mp4'),
    );

ActivityItem sampleActivity() {
  return ActivityItem(
    id: ActivityId.parse('activity-1'),
    type: ActivityType.publish,
    description: ActivityDescription(
      title: 'Published a video',
      body: 'A relay-side banger',
    ),
    occurredAt: DateTime(2026, 3, 12, 10),
  );
}
