import 'package:flutter/material.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class VideoCommentTile extends StatelessWidget {
  const VideoCommentTile({
    required this.comment,
    required this.onReply,
    super.key,
  });

  final VideoComment comment;
  final VoidCallback? onReply;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(
        left: comment.isReply ? AppSpacing.replyIndent : 0,
      ),
      child: ListTile(
        title: Text(comment.authorLabel),
        subtitle: Text(comment.content),
        trailing: IconButton(
          tooltip: 'Reply to ${comment.authorLabel}',
          onPressed: onReply,
          icon: const Icon(Icons.reply),
        ),
      ),
    );
  }
}
