Pod::Spec.new do |s|
  s.name             = 'rust_lib_ghostr'
  s.version          = '0.0.1'
  s.summary          = 'Ghostr native video gateway.'
  s.description      = <<-DESC
Ghostr native video gateway.
                       DESC
  s.homepage         = 'https://github.com/ghostr-social'
  s.license          = { :type => 'MIT' }
  s.author           = { 'Ghostr' => 'opensource@ghostr.social' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'FlutterMacOS'
  s.platform = :osx, '10.11'
  s.swift_version = '5.0'

  s.script_phase = {
    :name => 'Build Rust library',
    :script => 'sh "$PODS_TARGET_SRCROOT/../cargokit/build_pod.sh" ../../rust rust_lib_ghostr',
    :execution_position => :before_compile,
    :input_files => ['${BUILT_PRODUCTS_DIR}/cargokit_phony'],
    :output_files => ['${BUILT_PRODUCTS_DIR}/librust_lib_ghostr.a'],
  }
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'OTHER_LDFLAGS' => '-force_load ${BUILT_PRODUCTS_DIR}/librust_lib_ghostr.a',
  }
end
