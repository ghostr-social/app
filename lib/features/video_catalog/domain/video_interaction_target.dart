import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class VideoInteractionTarget {
  factory VideoInteractionTarget.fromPost(VideoPost post) {
    final reference = post.nostrReference;
    final identifier = reference?.identifier;
    final kind = reference?.kind.value;
    if (identifier != null && kind != null && kind >= 30000 && kind < 40000) {
      return VideoInteractionTarget._(
        'a:$kind:${reference!.authorPublicKeyHex.value}:${identifier.value}',
      );
    }
    return VideoInteractionTarget._(
      'e:${reference?.eventId.value ?? post.id.value}',
    );
  }

  const VideoInteractionTarget._(this._value);

  final String _value;

  @override
  bool operator ==(Object other) {
    return other is VideoInteractionTarget && other._value == _value;
  }

  @override
  int get hashCode => _value.hashCode;
}
