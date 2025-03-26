{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.lld_18
    pkgs.llvmPackages_18.llvm
    pkgs.libxml2
    pkgs.libffi
    pkgs.pkg-config
    pkgs.glibc
  ];

  # Set environment variables to help find libraries
  LLVM_SYS_180_PREFIX = "${pkgs.llvmPackages_18.llvm.dev}";
}