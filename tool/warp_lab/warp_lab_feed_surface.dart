import 'package:flutter/material.dart';

import '../../integration_test/support/warp_feed_production_graph.dart';
import '../../integration_test/support/warp_feed_surface.dart';
import 'warp_lab_destination.dart';

class WarpLabFeedSurface extends StatelessWidget {
  const WarpLabFeedSurface({
    required this.destination,
    required this.graph,
    super.key,
  });

  final WarpLabDestination destination;
  final WarpFeedProductionGraph graph;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      label: destination.semanticLabel,
      child: WarpFeedSurface(
        graph: graph,
        overlay: ExcludeSemantics(child: _routeBadge(context)),
      ),
    );
  }

  Widget _routeBadge(BuildContext context) {
    return IgnorePointer(
      child: SafeArea(
        child: Align(
          alignment: Alignment.topLeft,
          child: Container(
            margin: const EdgeInsets.all(12),
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface.withAlpha(210),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Text(destination.title),
          ),
        ),
      ),
    );
  }
}
