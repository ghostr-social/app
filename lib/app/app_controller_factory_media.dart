part of 'app_controller_factory.dart';

extension AppControllerFactoryMedia on AppControllerFactory {
  VideoPlaybackPort get videoPlaybackPort => _dependencies.videoPlaybackPort;

  IncomingVideoSharePort get incomingVideoSharePort =>
      _dependencies.incomingVideoSharePort;

  VideoShareWorkflow get videoShareWorkflow => _dependencies.videoShareWorkflow;
}
