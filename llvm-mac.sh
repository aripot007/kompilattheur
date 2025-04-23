# Si les dépendances ne sont pas installées, les installer :
if ! brew list llvm@15 &>/dev/null; then
    brew install llvm@15
fi

if ! brew list lld &>/dev/null; then
    brew install lld
fi

# Exporter les variables d'environnement nécessaires
export LLVM_SYS_180_PREFIX=$(brew --prefix llvm@15)
export PATH="$LLVM_SYS_180_PREFIX/bin:$PATH"
export LIBRARY_PATH=/opt/homebrew/lib:$LIBRARY_PATH