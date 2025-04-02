# Si les dépendances ne sont pas installées, les installer :
brew install llvm@18 lld 

# Exporter les variables d'environnement nécessaires
export LLVM_SYS_180_PREFIX=$(brew --prefix llvm@18)
export PATH="$LLVM_SYS_180_PREFIX/bin:$PATH"
export LIBRARY_PATH=/opt/homebrew/lib:$LIBRARY_PATH