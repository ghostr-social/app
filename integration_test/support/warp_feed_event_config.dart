enum WarpFeedCandidateLayout { primaryOnly, nextWithRescue }

final class SignedWarpFeedConfig {
  const SignedWarpFeedConfig({
    this.eventCount = 3,
    this.candidateLayout = WarpFeedCandidateLayout.primaryOnly,
  }) : assert(eventCount > 0 && eventCount <= 10);

  final int eventCount;
  final WarpFeedCandidateLayout candidateLayout;

  WarpFeedEventSources sourcesFor(String label) {
    final fallback =
        candidateLayout == WarpFeedCandidateLayout.nextWithRescue &&
            label == 'next'
        ? 'next-rescue'
        : null;
    return WarpFeedEventSources(label, fallbackLabel: fallback);
  }
}

final class WarpFeedEventSources {
  const WarpFeedEventSources(this.primaryLabel, {this.fallbackLabel});

  final String primaryLabel;
  final String? fallbackLabel;
}
