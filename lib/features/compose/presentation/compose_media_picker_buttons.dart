part of 'compose_form.dart';

class _ComposeMediaPickerButtons extends StatelessWidget {
  const _ComposeMediaPickerButtons({
    required this.capabilities,
    required this.actions,
    required this.isBusy,
  });

  final MediaPickerCapabilities capabilities;
  final ComposeFormActions actions;
  final bool isBusy;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _libraryButton(),
        if (!capabilities.library) _unavailable(context, _libraryMessage),
        const SizedBox(height: AppSpacing.sm),
        _cameraButton(),
        if (!capabilities.camera) _unavailable(context, _cameraMessage),
      ],
    );
  }

  Widget _libraryButton() {
    return Semantics(
      label: capabilities.library
          ? 'Choose video from library'
          : 'Video library unavailable on this device',
      button: true,
      enabled: capabilities.library && !isBusy,
      excludeSemantics: true,
      child: ElevatedButton(
        onPressed: capabilities.library && !isBusy ? actions.onChoose : null,
        child: const Text('Choose from library'),
      ),
    );
  }

  Widget _cameraButton() {
    return Semantics(
      label: capabilities.camera
          ? 'Capture video'
          : 'Capture video unavailable on this device',
      button: true,
      enabled: capabilities.camera && !isBusy,
      excludeSemantics: true,
      child: FilledButton.tonal(
        onPressed: capabilities.camera && !isBusy ? actions.onCapture : null,
        child: const Text('Capture video'),
      ),
    );
  }

  Widget _unavailable(BuildContext context, String message) {
    return Padding(
      padding: const EdgeInsets.only(top: AppSpacing.sm),
      child: Text(
        message,
        textAlign: TextAlign.center,
        style: Theme.of(context).textTheme.bodySmall,
      ),
    );
  }

  String get _libraryMessage =>
      'Video library selection is unavailable on this device.';
  String get _cameraMessage => 'Video capture is unavailable on this device.';
}

extension on MediaPickerCapabilities {
  String get sourceLabel {
    if (library && camera) return 'your gallery or camera';
    if (library) return 'your gallery';
    if (camera) return 'your camera';
    return 'a supported device source';
  }
}
