import 'package:video_player/video_player.dart';


class UserData {
  final String? npub;
  final String? name;
  final String? profilePicture;
  const UserData({
    this.npub,
    this.name,
    this.profilePicture,
  });
}

class Video {
  String id;
  UserData user;
  String videoTitle;
  String songName;
  String likes;
  String comments;
  String url;

  VideoPlayerController? controller;

  Video(
      {required this.id,
      required this.user,
      required this.videoTitle,
      required this.songName,
      required this.likes,
      required this.comments,
      required this.url});

  Future<Null> loadController() async {
    print("loading ${url}");
    controller = VideoPlayerController.networkUrl(Uri.parse(url));
    await controller?.initialize();
    controller?.setLooping(true);
  }
}
