part of 'warp_evidence_models.dart';

enum WarpTransferRequestKind { range, whole }

typedef WarpPlanTransferData = ({
  String postId,
  String sourceId,
  WarpTransferRequestKind requestKind,
  int start,
  int end,
  String reason,
  int? actionId,
  int expectedDeliveryMs,
});

final class WarpPlanTransfer {
  const WarpPlanTransfer(this.value);

  final WarpPlanTransferData value;

  String get postId => value.postId;
  String get sourceId => value.sourceId;
  WarpTransferRequestKind get requestKind => value.requestKind;
  int get start => value.start;
  int get end => value.end;
  String get reason => value.reason;
  int? get actionId => value.actionId;
  int get expectedDeliveryMs => value.expectedDeliveryMs;
}
