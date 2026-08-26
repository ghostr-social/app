import 'package:flutter/material.dart';

import 'warp_lab_app.dart';
import 'warp_lab_destination.dart';

class WarpLabMenuApp extends StatelessWidget {
  const WarpLabMenuApp({required this.onOpen, super.key});

  final ValueChanged<WarpLabDestination> onOpen;

  @override
  Widget build(BuildContext context) {
    return WarpLabApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('WARP Lab')),
        body: ListView(
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Text(
                'Choose one route per app launch. Relaunch to switch routes.',
              ),
            ),
            ...WarpLabDestination.tests.map(_entry),
          ],
        ),
      ),
    );
  }

  Widget _entry(WarpLabDestination destination) {
    return Semantics(
      button: true,
      label: 'Open ${destination.title}',
      child: ListTile(
        title: Text(destination.title),
        subtitle: Text(destination.description),
        trailing: const Icon(Icons.chevron_right),
        onTap: () => onOpen(destination),
      ),
    );
  }
}
