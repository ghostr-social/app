import 'package:flutter/widgets.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

class FeedScreenBindings {
  const FeedScreenBindings({
    required this.onOpenProfile,
    required this.onOpenHashtag,
    required this.playbackPort,
    required this.shareWorkflow,
    required this.createComments,
    required this.isActive,
    this.showFeedKindSelector = false,
  });

  final ValueChanged<ProfileId> onOpenProfile;
  final ValueChanged<String> onOpenHashtag;
  final VideoPlaybackPort playbackPort;
  final VideoShareWorkflow shareWorkflow;
  final CommentsCubit Function(VideoPost post) createComments;
  final bool isActive;
  final bool showFeedKindSelector;
}
