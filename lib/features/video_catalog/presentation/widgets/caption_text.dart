import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class CaptionText extends StatefulWidget {
  const CaptionText({
    required this.caption,
    required this.onHashtagTap,
    this.style,
    this.maxLines,
    super.key,
  });

  final String caption;
  final ValueChanged<String> onHashtagTap;
  final TextStyle? style;
  final int? maxLines;

  @override
  State<CaptionText> createState() => _CaptionTextState();
}

class _CaptionTextState extends State<CaptionText> {
  final List<TapGestureRecognizer> _recognizers = [];
  late List<InlineSpan> _spans;

  @override
  void initState() {
    super.initState();
    _spans = _buildSpans();
  }

  @override
  void didUpdateWidget(CaptionText oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.caption != widget.caption ||
        oldWidget.onHashtagTap != widget.onHashtagTap) {
      _disposeRecognizers();
      _spans = _buildSpans();
    }
  }

  @override
  void dispose() {
    _disposeRecognizers();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Text.rich(
      TextSpan(style: widget.style, children: _spans),
      maxLines: widget.maxLines,
      overflow:
          widget.maxLines == null ? TextOverflow.clip : TextOverflow.ellipsis,
    );
  }

  List<InlineSpan> _buildSpans() {
    final spans = <InlineSpan>[];
    var cursor = 0;
    for (final match in hashtagPattern.allMatches(widget.caption)) {
      if (match.start > cursor) {
        spans.add(
          TextSpan(text: widget.caption.substring(cursor, match.start)),
        );
      }
      spans.add(_hashtagSpan(match.group(0)!));
      cursor = match.end;
    }
    if (cursor < widget.caption.length) {
      spans.add(TextSpan(text: widget.caption.substring(cursor)));
    }
    return spans;
  }

  TextSpan _hashtagSpan(String hashtag) {
    final recognizer = TapGestureRecognizer()
      ..onTap = () => widget.onHashtagTap(hashtag);
    _recognizers.add(recognizer);
    return TextSpan(
      text: hashtag,
      recognizer: recognizer,
      style: const TextStyle(
        color: AppPalette.accentBlue,
        fontWeight: FontWeight.w700,
      ),
    );
  }

  void _disposeRecognizers() {
    for (final recognizer in _recognizers) {
      recognizer.dispose();
    }
    _recognizers.clear();
  }
}
