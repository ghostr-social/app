import 'package:ghostr/features/video_inventory/domain/media_url_policy.dart';

class AllowAllMediaUrlPolicy implements MediaUrlPolicy {
  const AllowAllMediaUrlPolicy();

  @override
  Future<void> validate(Uri source) async {}
}
