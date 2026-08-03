import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comment_composer.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/comments/presentation/video_comment_tile.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class CommentsSheet extends StatefulWidget {
  const CommentsSheet({required this.onCommentPublished, super.key});

  final VoidCallback onCommentPublished;

  @override
  State<CommentsSheet> createState() => _CommentsSheetState();
}

class _CommentsSheetState extends State<CommentsSheet> {
  final _controller = TextEditingController();
  final _focusNode = FocusNode();

  @override
  Widget build(BuildContext context) {
    return BlocListener<CommentsCubit, CommentsState>(
      listenWhen: (_, state) => state.notice != null,
      listener: _showNotice,
      child: SizedBox(
        height: MediaQuery.sizeOf(context).height * 0.72,
        child: BlocBuilder<CommentsCubit, CommentsState>(builder: _content),
      ),
    );
  }

  Widget _content(BuildContext context, CommentsState state) {
    return Column(
      children: [
        _header,
        Expanded(child: _body(state)),
        if (_hasComposer(state)) _composer(state),
      ],
    );
  }

  static const _header = ListTile(
    title: Text('Comments'),
    leading: Icon(Icons.chat_bubble_outline),
  );

  Widget _body(CommentsState state) {
    return switch (state) {
      CommentsLoading() => const LoadingPanel(label: 'Loading comments'),
      CommentsFailure(:final failureMessage) => _errorPanel(failureMessage),
      CommentsContent(:final comments) when comments.isEmpty =>
        const Center(child: Text('No comments yet')),
      CommentsContent(:final comments, :final isPosting) =>
        _commentList(comments, isPosting),
    };
  }

  Widget _commentList(List<VideoComment> comments, bool isPosting) {
    return ListView.builder(
      itemCount: comments.length,
      itemBuilder: (_, index) => VideoCommentTile(
        comment: comments[index],
        onReply: isPosting ? null : () => _selectReply(comments[index]),
      ),
    );
  }

  Widget _composer(CommentsState state) {
    return CommentComposer(
      model: CommentComposerModel(
        controller: _controller,
        focusNode: _focusNode,
        replyTo: state.replyTo,
        isPosting: state.isPosting,
      ),
      onChanged: () => setState(() {}),
      onPublish: _publish,
    );
  }

  Widget _errorPanel(String message) {
    return AsyncStatePanel(
      icon: Icons.comments_disabled_outlined,
      title: 'Comments unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<CommentsCubit>().load,
    );
  }

  void _selectReply(VideoComment comment) {
    context.read<CommentsCubit>().selectReply(comment);
    _focusNode.requestFocus();
  }

  Future<void> _publish() async {
    final published =
        await context.read<CommentsCubit>().publish(_controller.text);
    if (!published) return;
    widget.onCommentPublished();
    if (mounted) _controller.clear();
  }

  void _showNotice(BuildContext context, CommentsState state) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(state.notice!)),
    );
    context.read<CommentsCubit>().clearNotice();
  }

  bool _hasComposer(CommentsState state) {
    return state is CommentsContent;
  }

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }
}
