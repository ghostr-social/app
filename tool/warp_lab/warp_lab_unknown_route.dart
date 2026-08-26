import 'package:flutter/material.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';

import 'warp_lab_app.dart';

class WarpLabUnknownRouteApp extends StatelessWidget {
  const WarpLabUnknownRouteApp({
    required this.route,
    required this.onOpenLab,
    super.key,
  });

  final String route;
  final VoidCallback onOpenLab;

  @override
  Widget build(BuildContext context) {
    return WarpLabApp(
      home: Scaffold(
        body: Semantics(
          container: true,
          label: 'Unknown WARP Lab route',
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              AsyncStatePanel(
                icon: Icons.route_outlined,
                title: 'Unknown WARP Lab route',
                message: 'No executable WARP test uses this route.',
                actionLabel: 'Open WARP Lab',
                onAction: onOpenLab,
              ),
              Text(route),
            ],
          ),
        ),
      ),
    );
  }
}
