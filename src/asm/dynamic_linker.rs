use inkwell::targets::TargetMachine;

/// Returns a dynamic linker path based on the target machine's triple.
///
/// This function replicates Clang's Linux::getDynamicLinker functionality.
pub fn get_dynamic_linker(target_machine: &TargetMachine) -> String {
    // Get the target triple as a string.
    let triple = target_machine.get_triple();
    let triple_str = triple.to_string();
    
    // Helper function to detect architecture
    let is_arch = |arch: &str| -> bool {
        triple_str.starts_with(arch) || triple_str.contains(&format!("{}", arch))
    };

    // Android handling
    if triple_str.contains("android") {
        return if triple_str.contains("aarch64") || 
                  triple_str.contains("x86_64") ||
                  triple_str.contains("mips64") {
            "/system/bin/linker64".to_string()
        } else {
            "/system/bin/linker".to_string()
        };
    }

    // Musl handling
    if triple_str.contains("musl") {
        let mut arch_name = String::new();
        
        if is_arch("arm") || is_arch("thumb") {
            // Handle ARM architectures
            if triple_str.contains("eb") {
                arch_name.push_str("armeb");
            } else {
                arch_name.push_str("arm");
            }
            
            // Check for hard-float
            if triple_str.contains("hf") || triple_str.contains("eabihf") {
                arch_name.push_str("hf");
            }
        } else if is_arch("aarch64") {
            if triple_str.contains("_be") {
                arch_name.push_str("aarch64_be");
            } else {
                arch_name.push_str("aarch64");
            }
        } else if is_arch("mips") || is_arch("mipsel") || is_arch("mips64") || is_arch("mips64el") {
            // Extract the exact MIPS architecture
            if triple_str.starts_with("mips64el") {
                arch_name.push_str("mips64el");
            } else if triple_str.starts_with("mips64") {
                arch_name.push_str("mips64");
            } else if triple_str.starts_with("mipsel") {
                arch_name.push_str("mipsel");
            } else {
                arch_name.push_str("mips");
            }
        } else {
            // For other architectures, use the first component of the triple
            arch_name.push_str(&triple_str.split('-').next().unwrap_or("generic"));
        }
        
        return format!("/lib/ld-musl-{}.so.1", arch_name);
    }

    // Standard Linux handling
    let (lib_dir, loader) = match true {
        _ if is_arch("aarch64") => {
            if triple_str.contains("_be") {
                ("lib".to_string(), "ld-linux-aarch64_be.so.1".to_string())
            } else {
                ("lib".to_string(), "ld-linux-aarch64.so.1".to_string())
            }
        },
        _ if is_arch("arm") || is_arch("thumb") => {
            let hf = triple_str.contains("hf") || triple_str.contains("eabihf");
            ("lib".to_string(), if hf { "ld-linux-armhf.so.3".to_string() } else { "ld-linux.so.3".to_string() })
        },
        _ if is_arch("mips64el") => {
            let nan2008 = triple_str.contains("nan2008");
            let lib_suffix = if triple_str.contains("n32") { "32" } else { "64" };
            
            if triple_str.contains("uclibc") {
                (
                    format!("lib{}", lib_suffix), 
                    if nan2008 { "ld-uClibc-mipsn8.so.0".to_string() } else { "ld-uClibc.so.0".to_string() }
                )
            } else {
                (
                    format!("lib{}", lib_suffix), 
                    if nan2008 { "ld-linux-mipsn8.so.1".to_string() } else { "ld.so.1".to_string() }
                )
            }
        },
        _ if is_arch("mips64") => {
            let nan2008 = triple_str.contains("nan2008");
            let lib_suffix = if triple_str.contains("n32") { "32" } else { "64" };
            
            if triple_str.contains("uclibc") {
                (
                    format!("lib{}", lib_suffix),
                    if nan2008 { "ld-uClibc-mipsn8.so.0".to_string() } else { "ld-uClibc.so.0".to_string() }
                )
            } else {
                (
                    format!("lib{}", lib_suffix),
                    if nan2008 { "ld-linux-mipsn8.so.1".to_string() } else { "ld.so.1".to_string() }
                )
            }
        },
        _ if is_arch("mipsel") => {
            let nan2008 = triple_str.contains("nan2008");
            
            if triple_str.contains("uclibc") {
                ("lib".to_string(), if nan2008 { "ld-uClibc-mipsn8.so.0".to_string() } else { "ld-uClibc.so.0".to_string() })
            } else {
                ("lib".to_string(), if nan2008 { "ld-linux-mipsn8.so.1".to_string() } else { "ld.so.1".to_string() })
            }
        },
        _ if is_arch("mips") => {
            let nan2008 = triple_str.contains("nan2008");
            
            if triple_str.contains("uclibc") {
                ("lib".to_string(), if nan2008 { "ld-uClibc-mipsn8.so.0".to_string() } else { "ld-uClibc.so.0".to_string() })
            } else {
                ("lib".to_string(), if nan2008 { "ld-linux-mipsn8.so.1".to_string() } else { "ld.so.1".to_string() })
            }
        },
        _ if is_arch("ppc64le") => {
            let elfv1 = triple_str.contains("elfv1");
            ("lib64".to_string(), if elfv1 { "ld64.so.1".to_string() } else { "ld64.so.2".to_string() })
        },
        _ if is_arch("ppc64") => {
            let elfv2 = triple_str.contains("elfv2");
            ("lib64".to_string(), if elfv2 { "ld64.so.2".to_string() } else { "ld64.so.1".to_string() })
        },
        _ if is_arch("ppc") => {
            ("lib".to_string(), "ld.so.1".to_string())
        },
        _ if is_arch("riscv64") => {
            // Extract ABI name - simplified as we don't have Args
            let abi = if triple_str.contains("lp64d") {
                "lp64d"
            } else if triple_str.contains("lp64f") {
                "lp64f"
            } else {
                "lp64"
            };
            ("lib".to_string(), format!("ld-linux-riscv64-{}.so.1", abi))
        },
        _ if is_arch("riscv32") => {
            // Extract ABI name - simplified as we don't have Args
            let abi = if triple_str.contains("ilp32d") {
                "ilp32d"
            } else if triple_str.contains("ilp32f") {
                "ilp32f"
            } else {
                "ilp32"
            };
            ("lib".to_string(), format!("ld-linux-riscv32-{}.so.1", abi))
        },
        _ if is_arch("sparc64") || is_arch("sparcv9") => {
            ("lib64".to_string(), "ld-linux.so.2".to_string())
        },
        _ if is_arch("sparc") => {
            ("lib".to_string(), "ld-linux.so.2".to_string())
        },
        _ if is_arch("s390x") || is_arch("systemz") => {
            ("lib".to_string(), "ld64.so.1".to_string())
        },
        _ if is_arch("x86_64") => {
            let x32 = triple_str.contains("x32") || triple_str.contains("gnux32");
            (if x32 { "libx32".to_string() } else { "lib64".to_string() }, 
             if x32 { "ld-linux-x32.so.2".to_string() } else { "ld-linux-x86-64.so.2".to_string() })
        },
        _ if is_arch("i386") || is_arch("i486") || is_arch("i586") || is_arch("i686") || is_arch("x86") => {
            ("lib".to_string(), "ld-linux.so.2".to_string())
        },
        // Default fallback
        _ => ("lib".to_string(), "ld-linux.so.2".to_string()),
    };

    // Exherbo distribution special handling
    if triple_str.contains("exherbo") && 
       (!triple_str.contains("-") || triple_str.contains("-pc-")) {
        return format!("/usr/{}/lib/{}", triple_str, loader);
    }
    
    format!("/{}/{}", lib_dir, loader)
}