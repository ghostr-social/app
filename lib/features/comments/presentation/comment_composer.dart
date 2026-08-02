import 'package:flutter/material.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class CommentComposerModel {
  const CommentComposerModel({
    required this.controller,
    required this.focusNode,
    required this.replyTo,
    required this.isPosting,
  });

  final TextEditingController controller;
  final FocusNode focusNode;
  final VideoComment? replyTo;
  final bool isPosting;

  bool get canPublish => !isPosting && controller.text.trim().isNotEmpty;
}

class CommentComposer extends StatelessWidget {
  const CommentComposer({
    required this.model,
    required this.onChanged,
    required this.onPublish,
    super.key,
  });

  final CommentComposerModel model;
  final VoidCallback onChanged;
  final VoidCallback onPublish;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.sm),
        child: Row(
          children: [
            Expanded(child: _textField()),
            _sendButton(),
          ],
        ),
      ),
    );
  }

  Widget _textField() {
    final replyTo = model.replyTo;
    return TextField(
      controller: model.controller,
      focusNode: model.focusNode,
      enabled: !model.isPosting,
      onChanged: (_) => onChanged(),
      decoration: InputDecoration(
        labelText: replyTo == null
            ? 'Add a comment'
            : 'Reply to ${replyTo.authorLabel}',
      ),
    );
  }

  Widget _sendButton() {
    return IconButton(
      tooltip: model.replyTo == null ? 'Post comment' : 'Post reply',
      onPressed: model.canPublish ? onPublish : null,
      icon: const Icon(Icons.send),
    );
  }
}
