import 'package:flutter/material.dart';

class SettingsUrlDialogRequest {
  const SettingsUrlDialogRequest({
    required this.title,
    required this.fieldKey,
    required this.hintText,
  });

  final String title;
  final Key fieldKey;
  final String hintText;
}

Future<String?> showSettingsUrlDialog(
  BuildContext context,
  SettingsUrlDialogRequest request,
) {
  final controller = TextEditingController();
  return showDialog<String>(
    context: context,
    builder: (context) => _dialog(context, controller, request),
  );
}

AlertDialog _dialog(
  BuildContext context,
  TextEditingController controller,
  SettingsUrlDialogRequest request,
) {
  return AlertDialog(
    title: Text(request.title),
    content: TextField(
      key: request.fieldKey,
      controller: controller,
      keyboardType: TextInputType.url,
      decoration: InputDecoration(hintText: request.hintText),
    ),
    actions: _actions(context, controller),
  );
}

List<Widget> _actions(
  BuildContext context,
  TextEditingController controller,
) {
  return [
    TextButton(
      onPressed: () => Navigator.pop(context),
      child: const Text('Cancel'),
    ),
    FilledButton(
      onPressed: () => Navigator.pop(context, controller.text),
      child: const Text('Add'),
    ),
  ];
}
