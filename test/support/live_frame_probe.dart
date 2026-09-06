import 'package:flutter/widgets.dart';

class LiveFrameProbe extends StatefulWidget {
  const LiveFrameProbe({required this.onBuild, super.key});
  final VoidCallback onBuild;

  @override
  State<LiveFrameProbe> createState() => _LiveFrameProbeState();
}

class _LiveFrameProbeState extends State<LiveFrameProbe>
    with SingleTickerProviderStateMixin {
  late final _animation = AnimationController(
    vsync: this,
    duration: const Duration(seconds: 1),
  )..repeat();

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: _animation,
    builder: (_, _) {
      widget.onBuild();
      return ColoredBox(
        color: Color.lerp(
          const Color(0xff000000),
          const Color(0xffffffff),
          _animation.value,
        )!,
      );
    },
  );

  @override
  void dispose() {
    _animation.dispose();
    super.dispose();
  }
}
