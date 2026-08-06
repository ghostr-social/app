import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/features/video_sharing/presentation/video_share_cubit.dart';

class VideoShareFeedScope extends StatelessWidget {
  const VideoShareFeedScope({
    required this.workflow,
    required this.child,
    super.key,
  });

  final VideoShareWorkflow workflow;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => VideoShareCubit(workflow),
      child: BlocListener<VideoShareCubit, VideoShareState>(
        listenWhen: (_, current) => current is VideoShareFailed,
        listener: _showFailure,
        child: child,
      ),
    );
  }

  void _showFailure(BuildContext context, VideoShareState state) {
    final failure = state as VideoShareFailed;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(failure.message)));
    context.read<VideoShareCubit>().clearFailure();
  }
}
