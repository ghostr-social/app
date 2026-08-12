import 'package:flutter/material.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';

part 'compose_media_picker_buttons.dart';

class ComposeFormModel {
  const ComposeFormModel({
    required this.media,
    required this.isPublishing,
    required this.errorMessage,
    this.isSelecting = false,
  });

  final VideoMediaSource? media;
  final bool isPublishing;
  final bool isSelecting;
  final String? errorMessage;
}

class ComposeFormActions {
  const ComposeFormActions({
    required this.onChoose,
    required this.onCapture,
    required this.onPublish,
  });

  final VoidCallback onChoose;
  final VoidCallback onCapture;
  final VoidCallback onPublish;
}

class ComposeFormBindings {
  const ComposeFormBindings({
    required this.captionController,
    required this.playbackPort,
    required this.isActive,
    required this.pickerCapabilities,
    this.onMediaReleased,
  });

  final TextEditingController captionController;
  final VideoPlaybackPort playbackPort;
  final bool isActive;
  final MediaPickerCapabilities pickerCapabilities;
  final VoidCallback? onMediaReleased;
}

class ComposeForm extends StatelessWidget {
  const ComposeForm({
    required this.model,
    required this.actions,
    required this.bindings,
    super.key,
  });

  final ComposeFormModel model;
  final ComposeFormActions actions;
  final ComposeFormBindings bindings;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(AppSpacing.lg),
        child: _form(context),
      ),
    );
  }

  Widget _form(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _header(context),
        const SizedBox(height: AppSpacing.xl),
        _mediaButtons(),
        const SizedBox(height: AppSpacing.xl),
        _draftPreview(),
        _captionField(),
        const SizedBox(height: AppSpacing.md),
        _publishButton(),
      ],
    );
  }

  Widget _header(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Create', style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: AppSpacing.sm),
        Text(
          'Publish a new clip from $_mediaSourceLabel into your Ghostr profile.',
          style: Theme.of(
            context,
          ).textTheme.bodyLarge?.copyWith(color: AppPalette.mutedForeground),
        ),
      ],
    );
  }

  Widget _mediaButtons() {
    return _ComposeMediaPickerButtons(
      capabilities: bindings.pickerCapabilities,
      actions: actions,
      isBusy: _isBusy,
    );
  }

  Widget _draftPreview() {
    final media = model.media;
    if (media == null) return _emptyDraft();
    return Column(
      children: [
        AspectRatio(
          aspectRatio: 9 / 16,
          child: ClipRRect(
            borderRadius: BorderRadius.circular(AppRadius.media),
            child: bindings.playbackPort.buildSurface(
              media: media,
              isActive: bindings.isActive,
              onPlaybackMediaReleased: bindings.onMediaReleased,
            ),
          ),
        ),
        const SizedBox(height: AppSpacing.md),
      ],
    );
  }

  Widget _emptyDraft() {
    return const AsyncStatePanel(
      icon: Icons.video_library_outlined,
      title: 'No draft selected',
      message: 'Pick a video to preview it, add a caption, and publish it.',
    );
  }

  Widget _captionField() {
    return TextField(
      key: const Key('compose-caption-field'),
      controller: bindings.captionController,
      enabled: !_isBusy,
      maxLines: 3,
      decoration: InputDecoration(
        labelText: 'Caption',
        errorText: model.errorMessage,
      ),
    );
  }

  Widget _publishButton() {
    return ElevatedButton(
      onPressed: _isBusy || model.media == null ? null : actions.onPublish,
      child: Text(model.isPublishing ? 'Publishing...' : 'Publish'),
    );
  }

  bool get _isBusy => model.isPublishing || model.isSelecting;
  String get _mediaSourceLabel => bindings.pickerCapabilities.sourceLabel;
}
