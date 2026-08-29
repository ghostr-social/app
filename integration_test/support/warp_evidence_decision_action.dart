part of 'warp_evidence_models.dart';

final class WarpDecisionAction {
  const WarpDecisionAction({
    required this.plannerActionId,
    required this.postId,
    required this.kind,
    required this.command,
    required this.sourceId,
    required this.targetActionId,
    required this.start,
    required this.end,
  });

  factory WarpDecisionAction.fromJson(Map<String, Object?> json) {
    final kind = _warpChild(json, 'kind');
    final command = _warpChild(json, 'command');
    final range = _warpActionRange(kind);
    return WarpDecisionAction(
      plannerActionId: _warpInt(json, 'planner_action_id'),
      postId: _warpString(json, 'post_id'),
      kind: _warpString(kind, 'kind'),
      command: _warpString(command, 'command'),
      sourceId: _warpCommandSource(command),
      targetActionId: _warpTargetActionId(command),
      start: range?.start,
      end: range?.end,
    );
  }

  final int plannerActionId;
  final String postId;
  final String kind;
  final String command;
  final String? sourceId;
  final int? targetActionId;
  final int? start;
  final int? end;
}

({int start, int end})? _warpActionRange(Map<String, Object?> json) {
  if (json.containsKey('bytes_start')) {
    return (
      start: _warpInt(json, 'bytes_start'),
      end: _warpInt(json, 'bytes_end'),
    );
  }
  if (json.containsKey('maximum_bytes')) {
    return (start: 0, end: _warpInt(json, 'maximum_bytes'));
  }
  return null;
}

String? _warpCommandSource(Map<String, Object?> json) {
  if (json['source_id'] case final String source) return source;
  final transfer = json['transfer'];
  return transfer == null
      ? null
      : _warpString(_warpObject(transfer, 'transfer'), 'source_id');
}

int? _warpTargetActionId(Map<String, Object?> command) {
  if (command.containsKey('action_id')) return _warpInt(command, 'action_id');
  if (command.containsKey('primary_action_id')) {
    return _warpInt(command, 'primary_action_id');
  }
  return null;
}
