import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/compose/presentation/compose_form.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

class ComposeScreen extends StatefulWidget {
  const ComposeScreen({
    required this.session,
    required this.playbackPort,
    required this.isActive,
    super.key,
  });

  final UserSession session;
  final VideoPlaybackPort playbackPort;
  final bool isActive;

  @override
  State<ComposeScreen> createState() => _ComposeScreenState();
}

class _ComposeScreenState extends State<ComposeScreen> {
  final _captionController = TextEditingController();

  @override
  void dispose() {
    _captionController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<ComposeCubit, ComposeState>(
      listenWhen: (_, state) => state.notice != null,
      listener: _showNotice,
      builder: _form,
    );
  }

  Widget _form(BuildContext context, ComposeState state) {
    final selected = state.media;
    final compose = context.read<ComposeCubit>();
    return ComposeForm(
      model: ComposeFormModel(
        media: selected == null ? null : VideoMediaSource.local(selected.path),
        isPublishing: state.isPublishing,
        isSelecting: state.isSelecting,
        errorMessage: state.errorMessage,
      ),
      actions: ComposeFormActions(
        onChoose: compose.chooseFromGallery,
        onCapture: compose.captureVideo,
        onPublish: _publish,
      ),
      bindings: ComposeFormBindings(
        captionController: _captionController,
        playbackPort: widget.playbackPort,
        isActive: widget.isActive,
        pickerCapabilities: compose.pickerCapabilities,
      ),
    );
  }

  void _publish() {
    context.read<ComposeCubit>().publish(
          widget.session,
          _captionController.text,
        );
  }

  void _showNotice(BuildContext context, ComposeState state) {
    _captionController.clear();
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(state.notice!)),
    );
    context.read<ComposeCubit>().clearNotice();
  }
}
