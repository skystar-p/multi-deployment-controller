{
  lib,
  rustPlatform,
  ...
}:
let
  fs = lib.fileset;
  sourceFiles = fs.gitTracked ./.;
in
rustPlatform.buildRustPackage {
  pname = "multi-deployment-controller";
  version = "0.0.1";

  src = fs.toSource {
    root = ./.;
    fileset = sourceFiles;
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
