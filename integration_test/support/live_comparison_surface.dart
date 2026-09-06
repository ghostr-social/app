import 'package:flutter/material.dart';

final class LiveComparisonSurface extends StatelessWidget {
  const LiveComparisonSurface({
    required this.label,
    required this.host,
    required this.child,
    super.key,
  });

  final String label;
  final String host;
  final Widget child;

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: ThemeData.dark(),
    home: Scaffold(
      appBar: AppBar(title: Text('Phone video test — $label')),
      body: Center(child: child),
      bottomNavigationBar: SafeArea(
        child: Padding(padding: const EdgeInsets.all(16), child: Text(host)),
      ),
    ),
  );
}
