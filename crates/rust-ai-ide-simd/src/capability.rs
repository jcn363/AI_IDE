/// Runtime SIMD capability detection for optimal instruction set selection
use std::collections::HashSet;

use crate::error::{SIMDError, SIMDResult};

/// Comprehensive SIMD capability information for the current platform
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SIMDCapabilities {
    /// Whether SIMD is supported at all
    pub has_simd: bool,
    /// SSE instruction support
    pub has_sse: bool,
    /// SSE2 instruction support
    pub has_sse2: bool,
    /// SSE3 instruction support
    pub has_sse3: bool,
    /// SSSE3 instruction support
    pub has_ssse3: bool,
    /// SSE4.1 support
    pub has_sse41: bool,
    /// SSE4.2 support
    pub has_sse42: bool,
    /// AVX instruction support
    pub has_avx: bool,
    /// AVX2 instruction support
    pub has_avx2: bool,
    /// AVX512 foundation support
    pub has_avx512f: bool,
    /// AVX512 vector neural network instructions
    pub has_avx512vnni: bool,
    /// AVX512 byte and word instructions
    pub has_avx512bw: bool,
    /// AVX512 conflict detection instructions
    pub has_avx512cd: bool,
    /// AVX512 doubleword and quadword instructions
    pub has_avx512dq: bool,
    /// AVX512 exponential and reciprocal instructions
    pub has_avx512er: bool,
    /// AVX512 prefetch instructions
    pub has_avx512pf: bool,
    /// AVX512 vector length extensions
    pub has_avx512vl: bool,
    /// FMA (Fused Multiply-Add) support
    pub has_fma: bool,
    /// AES instruction support
    pub has_aes: bool,
    /// PCLMULQDQ support
    pub has_pclmulqdq: bool,
    /// BMI (Bit Manipulation Instructions)
    pub has_bmi1: bool,
    pub has_bmi2: bool,
    /// LZCNT (Leading Zero Count)
    pub has_lzcnt: bool,
    /// POPCNT (Population Count)
    pub has_popcnt: bool,
    /// TSX (Transactional Synchronization Extensions)
    pub has_tsx: bool,
    /// RDSEED random number generation
    pub has_rdseed: bool,
    /// RDPID (Read Processor ID)
    pub has_rdpid: bool,
    /// SHA (Secure Hash Algorithm) extensions
    pub has_sha: bool,
    /// UMIP (User-Mode Instruction Prevention)
    pub has_umip: bool,
    /// Mitigations enabled (Spectre/Meltdown)
    pub has_mitigations: bool,
    /// Maximum vector width in bytes
    pub max_vector_width: usize,
    /// Preferred vector alignment
    pub preferred_alignment: usize,
    /// Supported data types for SIMD operations
    pub supported_data_types: HashSet<String>,
}

impl Default for SIMDCapabilities {
    fn default() -> Self {
        Self {
            has_simd: false,
            has_sse: false,
            has_sse2: false,
            has_sse3: false,
            has_ssse3: false,
            has_sse41: false,
            has_sse42: false,
            has_avx: false,
            has_avx2: false,
            has_avx512f: false,
            has_avx512vnni: false,
            has_avx512bw: false,
            has_avx512cd: false,
            has_avx512dq: false,
            has_avx512er: false,
            has_avx512pf: false,
            has_avx512vl: false,
            has_fma: false,
            has_aes: false,
            has_pclmulqdq: false,
            has_bmi1: false,
            has_bmi2: false,
            has_lzcnt: false,
            has_popcnt: false,
            has_tsx: false,
            has_rdseed: false,
            has_rdpid: false,
            has_sha: false,
            has_umip: false,
            has_mitigations: false,
            max_vector_width: 1, // Scalar fallback
            preferred_alignment: 8,
            supported_data_types: HashSet::from(["f32".into(), "i32".into()]),
        }
    }
}

impl SIMDCapabilities {
    /// Check if any SIMD instruction set is available
    pub fn has_simd(&self) -> bool {
        self.has_sse || self.has_avx || self.has_avx2 || self.has_avx512f
    }

    /// Check if accelerated matrix operations are supported
    pub fn has_acclerated_matrix_ops(&self) -> bool {
        self.has_fma && (self.has_sse || self.has_avx || self.has_avx2)
    }

    /// Get optimal vector size for a specific data type
    pub fn vector_size_for_type<T>(&self) -> usize {
        if !self.has_simd {
            return 1;
        }

        std::mem::size_of::<T>()
            * match std::mem::size_of::<T>() {
                1 => 32,                     // i8/u8
                2 => 16,                     // i16/u16
                4 if self.has_avx512f => 16, // f32/i32 with AVX512
                4 if self.has_avx2 => 8,     // f32/i32 with AVX2
                4 if self.has_sse => 4,      // f32/i32 with SSE
                8 if self.has_avx512f => 8,  // f64/i64 with AVX512
                8 if self.has_sse2 => 2,     // f64/i64 with SSE2
                _ => 1,
            }
            .min(self.max_vector_width / std::mem::size_of::<T>())
    }

    /// Check if a specific data type is supported for SIMD operations
    pub fn is_data_type_supported(&self, type_name: &str) -> bool {
        self.supported_data_types.contains(type_name)
    }

    /// Get the maximum SIMD vector width in elements for a given type
    pub fn max_vector_elements<T>(&self) -> usize {
        if !self.has_simd {
            return 1;
        }
        self.max_vector_width / std::mem::size_of::<T>()
    }

    /// Get recommended alignment for SIMD operations
    pub fn recommended_alignment(&self) -> usize {
        if self.has_avx512f {
            64 // Cache line aligned for AVX512
        } else if self.has_avx || self.has_avx2 {
            32 // AVX alignment
        } else if self.has_sse {
            16 // SSE alignment
        } else {
            self.preferred_alignment
        }
    }

    /// Check if we can use wide SIMD vectors efficiently
    pub fn can_use_wide_vectors(&self) -> bool {
        self.max_vector_width >= 32 // AVX or above
    }

    /// Get a human-readable description of available SIMD features
    pub fn describe_capabilities(&self) -> String {
        if !self.has_simd {
            return "SIMD not available (scalar operations only)".to_string();
        }

        let mut features = Vec::new();
        if self.has_sse {
            features.push("SSE");
        }
        if self.has_sse2 {
            features.push("SSE2");
        }
        if self.has_sse3 {
            features.push("SSE3");
        }
        if self.has_ssse3 {
            features.push("SSSE3");
        }
        if self.has_sse41 {
            features.push("SSE4.1");
        }
        if self.has_sse42 {
            features.push("SSE4.2");
        }
        if self.has_avx {
            features.push("AVX");
        }
        if self.has_avx2 {
            features.push("AVX2");
        }
        if self.has_fma {
            features.push("FMA");
        }
        if self.has_avx512f {
            features.push("AVX512");
            if self.has_avx512vnni {
                features.push("VNNI");
            }
            if self.has_avx512bw {
                features.push("BW");
            }
            if self.has_avx512cd {
                features.push("CD");
            }
            if self.has_avx512dq {
                features.push("DQ");
            }
            if self.has_avx512vl {
                features.push("VL");
            }
        }

        format!(
            "SIMD available: {} (max vector width: {} bytes)",
            features.join("+"),
            self.max_vector_width
        )
    }
}

/// Detect SIMD capabilities at runtime using CPUID
pub fn detect_simd_capabilities() -> SIMDResult<SIMDCapabilities> {
    // Start with default (no SIMD) and build up from there
    let mut caps = SIMDCapabilities::default();

    // Try to detect CPUID features
    if let Err(e) = detect_cpu_features(&mut caps) {
        tracing::warn!("CPU feature detection failed: {:?}", e);
        // Return best-effort capabilities instead of failing
        return Ok(caps);
    }

    // Update supported data types based on detected features
    update_supported_data_types(&mut caps);

    tracing::info!(
        "Detected SIMD capabilities: {}",
        caps.describe_capabilities()
    );

    Ok(caps)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn detect_cpu_features(caps: &mut SIMDCapabilities) -> SIMDResult<()> {
    use std::arch::x86_64 as arch;

    // Check if we can safely use cpuid
    if !std::is_x86_feature_detected!("sse") {
        return Ok(()); // No SIMD available
    }

    caps.has_simd = true;

    // Basic CPU feature detection
    caps.has_sse = std::is_x86_feature_detected!("sse");
    caps.has_sse2 = std::is_x86_feature_detected!("sse2");
    caps.has_sse3 = std::is_x86_feature_detected!("sse3");
    caps.has_ssse3 = std::is_x86_feature_detected!("ssse3");
    caps.has_sse41 = std::is_x86_feature_detected!("sse4.1");
    caps.has_sse42 = std::is_x86_feature_detected!("sse4.2");
    caps.has_avx = std::is_x86_feature_detected!("avx");
    caps.has_avx2 = std::is_x86_feature_detected!("avx2");
    caps.has_fma = std::is_x86_feature_detected!("fma");
    caps.has_aes = std::is_x86_feature_detected!("aes");
    caps.has_pclmulqdq = std::is_x86_feature_detected!("pclmulqdq");
    caps.has_bmi1 = std::is_x86_feature_detected!("bmi1");
    caps.has_bmi2 = std::is_x86_feature_detected!("bmi2");
    caps.has_lzcnt = std::is_x86_feature_detected!("lzcnt");
    caps.has_popcnt = std::is_x86_feature_detected!("popcnt");
    caps.has_sha = std::is_x86_feature_detected!("sha");

    // AVX512 features (only check if AVX512 foundation is available to avoid crashes)
    if std::is_x86_feature_detected!("avx512f") {
        caps.has_avx512f = true;
        caps.has_avx512vnni = std::is_x86_feature_detected!("avx512vnni");
        caps.has_avx512bw = std::is_x86_feature_detected!("avx512bw");
        caps.has_avx512cd = std::is_x86_feature_detected!("avx512cd");
        caps.has_avx512dq = std::is_x86_feature_detected!("avx512dq");
        caps.has_avx512er = std::is_x86_feature_detected!("avx512er");
        caps.has_avx512pf = std::is_x86_feature_detected!("avx512pf");
        caps.has_avx512vl = std::is_x86_feature_detected!("avx512vl");
    }

    // Set vector width based on highest supported SIMD level
    caps.max_vector_width = if caps.has_avx512f {
        64 // AVX512: 512 bits = 64 bytes
    } else if caps.has_avx || caps.has_avx2 {
        32 // AVX/AVX2: 256 bits = 32 bytes
    } else if caps.has_sse {
        16 // SSE: 128 bits = 16 bytes
    } else {
        1 // Scalar only
    };

    // Set preferred alignment
    caps.preferred_alignment = if caps.max_vector_width >= 32 { 32 } else { 16 };

    Ok(())
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn detect_cpu_features(_caps: &mut SIMDCapabilities) -> SIMDResult<()> {
    // Non-x86 architectures don't have SIMD yet, keep defaults
    Ok(())
}

fn update_supported_data_types(caps: &mut SIMDCapabilities) {
    let mut types = HashSet::from(["f32".into(), "i32".into()]);

    if caps.has_simd {
        types.insert("f64".into());
        types.insert("i64".into());
        types.insert("i16".into());
        types.insert("i8".into());
        types.insert("u32".into());
        types.insert("u16".into());
        types.insert("u8".into());
    }

    if caps.has_avx512f {
        // AVX512 supports complex types
        types.insert("f64x4".into());
        types.insert("f32x16".into());
        types.insert("i32x16".into());
    } else if caps.has_avx2 || caps.has_avx {
        types.insert("f64x2".into());
        types.insert("f32x8".into());
        types.insert("i32x8".into());
        types.insert("i16x16".into());
    } else if caps.has_sse2 {
        types.insert("f64x2".into());
        types.insert("i32x4".into());
        types.insert("i16x8".into());
    }

    caps.supported_data_types = types;
}

/// Cache SIMD capability detection to avoid repeated CPUID calls
static mut CAPABILITY_CACHE: Option<SIMDCapabilities> = None;
static CAPABILITY_INIT: std::sync::Once = std::sync::Once::new();

pub fn get_cached_capabilities() -> &'static SIMDCapabilities {
    unsafe {
        CAPABILITY_INIT.call_once(|| {
            CAPABILITY_CACHE = detect_simd_capabilities().ok();
        });
        CAPABILITY_CACHE
            .as_ref()
            .unwrap_or(&SIMDCapabilities::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_detection() {
        let caps = detect_simd_capabilities();
        assert!(caps.is_ok());
    }

    #[test]
    fn test_capability_caching() {
        let caps1 = get_cached_capabilities();
        let caps2 = get_cached_capabilities();

        // Should be identical (same instance or identical values)
        assert_eq!(caps1.has_simd, caps2.has_simd);
        assert_eq!(caps1.max_vector_width, caps2.max_vector_width);
    }

    #[test]
    fn test_capability_description() {
        let mut caps = SIMDCapabilities::default();
        caps.has_simd = true;
        caps.has_avx2 = true;
        caps.max_vector_width = 32;

        let desc = caps.describe_capabilities();
        assert!(desc.contains("AVX2"));
        assert!(desc.contains("32"));
    }

    #[test]
    fn test_vector_size_calculation() {
        let mut caps = SIMDCapabilities::default();
        caps.max_vector_width = 32; // AVX

        // 4 bytes * 8 elements = 32 bytes (fits)
        assert_eq!(caps.vector_size_for_type::<f32>(), 8);

        // 8 bytes * 4 elements = 32 bytes (fits)
        assert_eq!(caps.vector_size_for_type::<f64>(), 4);

        // 4 bytes * 16 elements = 64 bytes (doesn't fit)
        assert_eq!(caps.vector_size_for_type::<u32>(), 8);
    }

    #[test]
    fn test_data_type_support() {
        let mut caps = SIMDCapabilities::default();
        caps.has_simd = true;

        update_supported_data_types(&mut caps);

        assert!(caps.is_data_type_supported("f32"));
        assert!(caps.is_data_type_supported("f64"));
        assert!(caps.is_data_type_supported("i32"));
        assert!(caps.is_data_type_supported("f32x8"));
    }
}
