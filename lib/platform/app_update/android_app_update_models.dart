enum AndroidNetworkAccess { none, wifi, other }

enum AndroidInstallMode { userConfirmed, automatic }

enum AndroidInstallState { pendingUserAction, succeeded, failed }

final class AndroidInstalledApp {
  const AndroidInstalledApp({
    required this.packageName,
    required this.versionCode,
    required this.versionName,
    required this.sdkInt,
    required this.supportedAbis,
  });

  final String packageName;
  final int versionCode;
  final String versionName;
  final int sdkInt;
  final List<String> supportedAbis;
}

final class AndroidInstallRequest {
  const AndroidInstallRequest({
    required this.path,
    required this.expectedVersionCode,
    required this.mode,
  });

  final String path;
  final int expectedVersionCode;
  final AndroidInstallMode mode;
}

final class AndroidInstallStatus {
  const AndroidInstallStatus({
    required this.sessionId,
    required this.state,
    this.message,
  });

  final int sessionId;
  final AndroidInstallState state;
  final String? message;

  @override
  bool operator ==(Object other) {
    return other is AndroidInstallStatus &&
        other.sessionId == sessionId &&
        other.state == state &&
        other.message == message;
  }

  @override
  int get hashCode => Object.hash(sessionId, state, message);
}
