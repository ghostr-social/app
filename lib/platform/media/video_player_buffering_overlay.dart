part of 'video_player_playback_port.dart';

final class _BufferingOverlay extends StatelessWidget {
  const _BufferingOverlay();

  @override
  Widget build(BuildContext context) {
    return const ColoredBox(
      color: AppPalette.videoLoadingBackground,
      child: LoadingPanel(label: 'Buffering video'),
    );
  }
}
