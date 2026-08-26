enum WarpLabDestination {
  menu('/warp', 'WARP Lab', 'Choose a production-path Android video test.'),
  feedPlayback(
    '/warp/feed-playback',
    'Feed playback',
    'Signed feed, Rust gateway, cache, and native player.',
  ),
  rapidSwipes(
    '/warp/rapid-swipes',
    'Rapid swipes',
    'Seven videos with parallel preparation and a slower origin.',
  ),
  networkEvidence(
    '/warp/network-evidence',
    'Network evidence',
    'Signed feed with controlled delivery-network evidence.',
  );

  const WarpLabDestination(this.path, this.title, this.description);

  static const tests = [feedPlayback, rapidSwipes, networkEvidence];

  final String path;
  final String title;
  final String description;

  String get semanticLabel => 'WARP ${title.toLowerCase()} test feed';

  static WarpLabDestination? fromPath(String path) {
    if (path == '/' || path.isEmpty) return menu;
    for (final destination in values) {
      if (destination.path == path) return destination;
    }
    return null;
  }
}
