import 'package:ghostr/features/app_update/domain/update_installer_port.dart';

final class FakeUpdateInstallerPort implements UpdateInstallerPort {
  UpdateInstallPermission permission = UpdateInstallPermission.granted;
  List<UpdateInstallStatus> statuses = [UpdateInstallStatus.succeeded];
  Object? permissionFailure;
  Object? openFailure;
  Object? submitFailure;
  Object? statusFailure;
  int permissionCalls = 0;
  int openCalls = 0;
  int statusCalls = 0;
  final List<UpdateInstallRequest> requests = [];
  final List<UpdateInstallSession> replacedSessions = [];

  @override
  Future<UpdateInstallPermission> readPermission() async {
    permissionCalls += 1;
    if (permissionFailure != null) throw permissionFailure!;
    return permission;
  }

  @override
  Future<void> openPermissionSettings() async {
    openCalls += 1;
    if (openFailure != null) throw openFailure!;
  }

  @override
  Future<UpdateInstallSession> submit(UpdateInstallRequest request) async {
    if (submitFailure != null) throw submitFailure!;
    requests.add(request);
    return UpdateInstallSession(requests.length);
  }

  @override
  Future<UpdateInstallSession> replace(
    UpdateInstallSession previous,
    UpdateInstallRequest request,
  ) async {
    replacedSessions.add(previous);
    return submit(request);
  }

  @override
  Future<UpdateInstallStatus> readStatus(UpdateInstallSession session) async {
    statusCalls += 1;
    if (statusFailure != null) throw statusFailure!;
    if (statuses.length == 1) return statuses.single;
    return statuses.removeAt(0);
  }
}
