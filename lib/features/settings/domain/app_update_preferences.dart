enum UpdateDownloadPolicy {
  manual,
  wifiOnly,
  anyNetwork;

  String get label => switch (this) {
    manual => 'Off',
    wifiOnly => 'Wi-Fi only',
    anyNetwork => 'Wi-Fi or mobile data',
  };
}

final class AppUpdatePreferences {
  const AppUpdatePreferences({
    required this.automaticChecks,
    required this.downloadPolicy,
    required this.automaticInstall,
  });

  static const defaults = AppUpdatePreferences(
    automaticChecks: true,
    downloadPolicy: UpdateDownloadPolicy.wifiOnly,
    automaticInstall: true,
  );

  final bool automaticChecks;
  final UpdateDownloadPolicy downloadPolicy;
  final bool automaticInstall;

  AppUpdatePreferences copyWith({
    bool? automaticChecks,
    UpdateDownloadPolicy? downloadPolicy,
    bool? automaticInstall,
  }) {
    if (automaticChecks == null &&
        downloadPolicy == null &&
        automaticInstall == null) {
      return this;
    }
    return AppUpdatePreferences(
      automaticChecks: automaticChecks ?? this.automaticChecks,
      downloadPolicy: downloadPolicy ?? this.downloadPolicy,
      automaticInstall: automaticInstall ?? this.automaticInstall,
    );
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        other is AppUpdatePreferences &&
            automaticChecks == other.automaticChecks &&
            downloadPolicy == other.downloadPolicy &&
            automaticInstall == other.automaticInstall;
  }

  @override
  int get hashCode =>
      Object.hash(automaticChecks, downloadPolicy, automaticInstall);
}
