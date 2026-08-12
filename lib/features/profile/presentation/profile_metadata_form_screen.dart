import 'package:flutter/material.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/presentation/profile_metadata_form_contract.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

export 'profile_metadata_form_contract.dart';

final class ProfileMetadataFormScreen extends StatefulWidget {
  const ProfileMetadataFormScreen({
    required this.configuration,
    required this.actions,
    this.viewState = const ProfileMetadataFormViewState(),
    super.key,
  });

  final ProfileMetadataFormConfiguration configuration;
  final ProfileMetadataFormActions actions;
  final ProfileMetadataFormViewState viewState;

  @override
  State<ProfileMetadataFormScreen> createState() => _ProfileFormState();
}

final class _ProfileFormState extends State<ProfileMetadataFormScreen> {
  late final _name = TextEditingController(text: _initial.displayName);
  late final _handle = TextEditingController(text: _initial.handle);
  late final _picture = TextEditingController(text: _initial.pictureUrl);
  String? _validationMessage;

  ProfileFormInitial get _initial => widget.configuration.initial;

  @override
  void initState() {
    super.initState();
    _name.addListener(_changed);
    _handle.addListener(_changed);
  }

  @override
  void dispose() {
    _name.dispose();
    _handle.dispose();
    _picture.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.configuration.title),
        leading: _backButton,
      ),
      body: ListView(
        padding: const EdgeInsets.all(AppSpacing.xl),
        children: _formChildren(context),
      ),
    );
  }

  Widget? get _backButton {
    if (widget.actions.onBack == null) return null;
    return IconButton(
      tooltip: 'Back',
      onPressed: _inputsEnabled ? widget.actions.onBack : null,
      icon: const Icon(Icons.arrow_back),
    );
  }

  List<Widget> _formChildren(BuildContext context) => [
    _field(_name, 'Name', 'profile-display-name-field'),
    const SizedBox(height: AppSpacing.md),
    _field(_handle, '@ handle', 'profile-handle-field'),
    const SizedBox(height: AppSpacing.md),
    _field(_picture, 'Picture URL (optional)', 'profile-picture-url-field'),
    ..._picturePickerChildren,
    ..._messageChildren(context),
    const SizedBox(height: AppSpacing.xl),
    ElevatedButton(
      key: widget.configuration.submitKey,
      onPressed: _canSubmit ? _submit : null,
      child: Text(_submitLabel),
    ),
  ];

  List<Widget> get _picturePickerChildren {
    if (widget.actions.onSelectPicture == null) return const [];
    return [
      const SizedBox(height: AppSpacing.sm),
      OutlinedButton.icon(
        key: const Key('profile-picture-picker'),
        onPressed: _inputsEnabled ? widget.actions.onSelectPicture : null,
        icon: const Icon(Icons.photo_library_outlined),
        label: Text(_pictureButtonLabel),
      ),
      if (widget.viewState.selectedPicture case final picture?)
        Text('Selected: ${picture.label}'),
    ];
  }

  List<Widget> _messageChildren(BuildContext context) {
    final message = _message;
    if (message == null) return const [];
    return [
      const SizedBox(height: AppSpacing.sm),
      Text(
        message,
        style: TextStyle(color: Theme.of(context).colorScheme.error),
      ),
    ];
  }

  Widget _field(TextEditingController controller, String label, String key) {
    return TextField(
      key: Key(key),
      controller: controller,
      enabled: _inputsEnabled,
      decoration: InputDecoration(labelText: label),
    );
  }

  bool get _inputsEnabled =>
      !widget.viewState.isSubmitting && !widget.viewState.isSelectingPicture;

  bool get _canSubmit =>
      _inputsEnabled &&
      _name.text.trim().isNotEmpty &&
      _handle.text.trim().isNotEmpty;

  String get _pictureButtonLabel => widget.viewState.isSelectingPicture
      ? 'Opening photos…'
      : 'Choose picture';

  String get _submitLabel => widget.viewState.isSubmitting
      ? 'Saving…'
      : widget.configuration.submitLabel;

  String? get _message => _validationMessage ?? widget.viewState.errorMessage;

  void _changed() => setState(() => _validationMessage = null);

  void _submit() {
    try {
      widget.actions.onSubmit(_parsedMetadata);
    } on FormatException catch (error) {
      setState(() => _validationMessage = error.message.toString());
    }
  }

  ProfileMetadata get _parsedMetadata => ProfileMetadata.parse(
    displayName: _name.text,
    handle: _handle.text,
    pictureUrl: _picture.text,
  );
}
