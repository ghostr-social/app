import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_sha256.dart';

/// Playable media resolved from a Nostr event: `imeta` tags first, then
/// direct video links written into the note text.
class NostrVideoMedia {
  const NostrVideoMedia({
    required this.urls,
    required this.delivery,
    this.expectedSha256,
  });

  final List<String> urls;
  final VideoMediaDelivery delivery;
  final String? expectedSha256;

  static NostrVideoMedia? fromEvent({
    required List<List<String>> tags,
    required String content,
  }) {
    return _fromImeta(tags) ?? _fromText(content);
  }

  static NostrVideoMedia? _fromImeta(List<List<String>> tags) {
    for (final tag in tags.where((tag) => tag.firstOrNull == 'imeta')) {
      final media = _tryImeta(tag);
      if (media != null) return media;
    }
    return null;
  }

  static NostrVideoMedia? _tryImeta(List<String> tag) {
    final urls = _imetaUrls(tag);
    final mimeType = _imetaField(tag, 'm');
    if (urls.isEmpty || !_playable(mimeType, urls.first)) return null;
    final digest = _imetaDigest(tag);
    if (!digest.valid) return null;
    return NostrVideoMedia(
      urls: urls,
      delivery: _imetaDelivery(mimeType, urls.first),
      expectedSha256: digest.value?.value,
    );
  }

  // Publishers often omit the mime; the URL extension is proof enough.
  static bool _playable(String? mimeType, String url) {
    if (mimeType != null) return _isVideoMime(mimeType);
    return _isVideoUrl(url);
  }

  static VideoMediaDelivery _imetaDelivery(String? mimeType, String url) {
    if (mimeType == null) return _urlDelivery(url);
    return _isHlsMime(mimeType)
        ? VideoMediaDelivery.hls
        : VideoMediaDelivery.progressive;
  }

  static ({bool valid, VideoSha256? value}) _imetaDigest(List<String> tag) {
    final rawDigest = _imetaField(tag, 'x');
    final value = rawDigest == null ? null : VideoSha256.tryParse(rawDigest);
    return (valid: rawDigest == null || value != null, value: value);
  }

  static List<String> _imetaUrls(List<String> tag) {
    final primary = _imetaField(tag, 'url');
    return <String>{
      if (_isHttpUrl(primary)) primary!,
      ..._imetaFallbacks(tag),
    }.toList();
  }

  static List<String> _imetaFallbacks(List<String> tag) {
    return tag
        .skip(1)
        .where((field) => field.startsWith('fallback '))
        .map((field) => field.substring('fallback '.length))
        .where(_isHttpUrl)
        .toList();
  }

  static String? _imetaField(List<String> tag, String name) {
    for (final field in tag.skip(1)) {
      if (field.startsWith('$name ')) return field.substring(name.length + 1);
    }
    return null;
  }

  static NostrVideoMedia? _fromText(String content) {
    for (final match in _linkPattern.allMatches(content)) {
      final url = _trimmedLink(match.group(0)!);
      if (_isVideoUrl(url)) {
        return NostrVideoMedia(urls: [url], delivery: _urlDelivery(url));
      }
    }
    return null;
  }

  static final RegExp _linkPattern = RegExp(r'https?://\S+');
  static const _trailingPunctuation = '.,;:!?)]}\'"';

  static String _trimmedLink(String raw) {
    var end = raw.length;
    while (end > 0 && _trailingPunctuation.contains(raw[end - 1])) {
      end -= 1;
    }
    return raw.substring(0, end);
  }

  static bool _isVideoUrl(String url) {
    return _isHttpUrl(url) && _urlExtension(url) != null;
  }

  static const _videoExtensions = ['.mp4', '.m4v', '.webm', '.mov', '.m3u8'];

  static String? _urlExtension(String url) {
    final path = Uri.tryParse(url)?.path.toLowerCase();
    if (path == null) return null;
    for (final extension in _videoExtensions) {
      if (path.endsWith(extension)) return extension;
    }
    return null;
  }

  static VideoMediaDelivery _urlDelivery(String url) {
    return _urlExtension(url) == '.m3u8'
        ? VideoMediaDelivery.hls
        : VideoMediaDelivery.progressive;
  }

  static bool _isHttpUrl(String? value) {
    final uri = value == null ? null : Uri.tryParse(value);
    return uri != null &&
        (uri.scheme == 'https' || uri.scheme == 'http') &&
        uri.host.isNotEmpty;
  }

  static bool _isVideoMime(String? value) {
    return _normalizedMime(value)?.startsWith('video/') == true ||
        _isHlsMime(value);
  }

  static bool _isHlsMime(String? value) {
    return const {
      'application/x-mpegurl',
      'application/vnd.apple.mpegurl',
    }.contains(_normalizedMime(value));
  }

  static String? _normalizedMime(String? value) => value?.trim().toLowerCase();
}

/// The caption should read as prose; links that became the playable media are
/// noise once the video renders.
String captionWithoutMediaUrls(String content, List<String> urls) {
  var caption = content;
  for (final url in urls) {
    caption = caption.replaceAll(url, ' ');
  }
  return caption.replaceAll(RegExp(r'\s+'), ' ').trim();
}
