import 'package:flutter_bloc/flutter_bloc.dart';

abstract class DisposalSafeCubit<State> extends Cubit<State> {
  DisposalSafeCubit(super.initialState);

  @override
  void emit(State state) {
    if (!isClosed) super.emit(state);
  }
}
