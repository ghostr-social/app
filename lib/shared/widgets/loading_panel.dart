import 'package:flutter/material.dart';

class LoadingPanel extends StatelessWidget {
  const LoadingPanel({required this.label, super.key});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: label,
      child: const Center(child: CircularProgressIndicator()),
    );
  }
}
