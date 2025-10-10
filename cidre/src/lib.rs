pub mod mac_types;

pub use mac_types::FourCharCode;
pub use mac_types::ResType;
pub use mac_types::UniChar;
pub use mac_types::four_cc_fmt_debug;
pub use mac_types::four_cc_to_str;
pub use mac_types::four_cc_to_string;

/// Apple Mobile
#[cfg(all(target_os = "macos", feature = "am"))]
pub mod am;

pub mod api;

pub mod arc;

/// Audio Toolkit
#[cfg(feature = "at")]
pub mod at;

/// AudioVisual Foundation (AVFoundation)
#[cfg(feature = "av")]
pub mod av;

/// Accessibility
#[cfg(all(target_os = "macos", feature = "ax"))]
pub mod ax;

/// cidre vision of obj-c blocks impl in rust
#[cfg(feature = "blocks")]
pub mod blocks;

/// Core Animation
#[cfg(not(target_os = "watchos"))]
#[cfg(feature = "ca")]
pub mod ca;

/// CoreAudioTypes
#[cfg(feature = "cat")]
pub mod cat;

/// Core Foundation
#[cfg(feature = "cf")]
pub mod cf;

/// Core Graphics
#[cfg(feature = "cg")]
pub mod cg;

/// Core Image
#[cfg(feature = "ci")]
pub mod ci;

/// Core Location
#[cfg(feature = "cl")]
pub mod cl;

/// Core Media
#[cfg(feature = "cm")]
pub mod cm;

#[cfg(all(target_os = "macos", feature = "core_audio"))]
pub mod core_audio;

/// Core Motion
#[cfg(not(target_os = "tvos"))]
#[cfg(feature = "core_motion")]
pub mod core_motion;

#[cfg(feature = "compression")]
pub mod compression;

/// Core Text
#[cfg(feature = "ct")]
pub mod ct;

/// Core Video
#[cfg(feature = "cv")]
pub mod cv;

/// Disk Arbitration
#[cfg(target_os = "macos")]
#[cfg(feature = "da")]
pub mod da;

/// Grand Central Dispatch
#[cfg(feature = "dispatch")]
pub mod dispatch;

pub mod dns_sd;

/// Game Controller
#[cfg(feature = "gc")]
pub mod gc;

pub mod io;

/// mach
pub mod mach;

/// MultipeerConnectivity
#[cfg(not(target_os = "watchos"))]
#[cfg(feature = "mc")]
pub mod mc;

#[cfg(feature = "mt")]
pub mod mt;

/// Metal
#[cfg(feature = "mtl")]
pub mod mtl;

#[cfg(feature = "mtl")]
pub mod mtl4;

/// MetalKit
#[cfg(feature = "mtk")]
pub mod mtk;

/// MLCompute
#[cfg(feature = "mlc")]
pub mod mlc;

/// Metal Performance Shaders
#[cfg(feature = "mps")]
pub mod mps;

#[cfg(feature = "ml")]
pub mod ml;

/// Foundation
#[cfg(feature = "ns")]
pub mod ns;

/// Natural Language
#[cfg(feature = "nl")]
pub mod nl;

/// Network
#[cfg(feature = "nw")]
pub mod nw;

#[cfg(feature = "ns")]
pub mod objc;

pub mod os;
pub mod sys;

/// Security
#[cfg(feature = "sec")]
pub mod sec;

/// Video Toolbox
#[cfg(feature = "vt")]
pub mod vt;

/// Accelerate vecLib vDSP
#[cfg(feature = "vdsp")]
pub mod vdsp;

#[cfg(feature = "vdsp")]
pub mod vimage;

#[cfg(feature = "cblas")]
mod cblas_new;
#[cfg(feature = "cblas")]
pub use cblas_new::catlas;
#[cfg(feature = "cblas")]
pub use cblas_new::cblas;

/// Screen Capture Kit
#[cfg(all(target_os = "macos", feature = "sc"))]
pub mod sc;

/// Sound Analysis
#[cfg(feature = "sn")]
pub mod sn;

#[cfg(all(
    any(
        target_os = "ios",
        all(target_os = "ios", target_abi = "macabi",),
        target_os = "tvos",
        target_os = "watchos",
        target_os = "visionos"
    ),
    feature = "ui"
))]
pub mod ui;

/// UniformTypeIdentifiers
#[cfg(feature = "ut")]
pub mod ut;

#[cfg(feature = "un")]
pub mod un;

pub mod time;

#[cfg(feature = "simd")]
pub mod simd;

/// Vision
#[cfg(feature = "vn")]
pub mod vn;

/// WatchConnectivity
#[cfg(target_os = "ios")]
#[cfg(feature = "wc")]
pub mod wc;

/// Web Kit
#[cfg(not(any(target_os = "tvos", target_os = "watchos")))]
#[cfg(feature = "wk")]
pub mod wk;

#[macro_export]
macro_rules! define_opts {
    (
        $(#[$outer:meta])*
        $vis:vis
        $NewType:ident($BaseType:path)
    ) => {
        $(#[$outer])*
        #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Default)]
        #[repr(transparent)]
        $vis struct $NewType(pub $BaseType);

        impl $NewType {
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0 == 0
            }

            pub fn any(&self, mask: Self) -> bool {
                self.0 & mask.0 != 0
            }

            #[inline]
            pub fn contains(&self, val: Self) -> bool {
                *self & val == val
            }

            #[inline]
            pub fn insert(&mut self, val: Self) {
                *self |= val
            }

            #[inline]
            pub fn remove(&mut self, val: Self) {
                *self &= !val
            }

            #[inline]
            pub fn set(&mut self, val: Self, on: bool) {
                if on {
                    self.insert(val)
                } else {
                    self.remove(val)
                }
            }
        }

        impl ::std::ops::BitAndAssign for $NewType {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = Self(self.0 & rhs.0)
            }
        }

        impl ::std::ops::BitAnd for $NewType {
            type Output = $NewType;

            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl ::std::ops::BitOr for $NewType {
            type Output = $NewType;

            #[inline]
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl ::std::ops::BitOrAssign for $NewType {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = Self(self.0 | rhs.0)
            }
        }

        impl ::std::ops::BitXor for $NewType {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl ::std::ops::BitXorAssign for $NewType {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0
            }
        }

        impl ::std::ops::Not for $NewType {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }

        impl ::std::convert::From<$BaseType> for $NewType {
            #[inline]
            fn from(value: $BaseType) -> Self {
                Self(value)
            }
        }

        impl ::std::fmt::Binary for $NewType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                ::std::fmt::Binary::fmt(&self.0, f)
            }
        }

    };
}

#[cfg(test)]
mod tests {
    use crate::cf;

    #[test]
    fn it_works() {
        let f = {
            let null = cf::Null::value();
            null.show();

            let num = cf::Number::from_i16(0);
            let arr = cf::Array::from_type_refs(&[null, &num]).unwrap();

            let v = b"he".to_vec();
            let _s = cf::String::create_with_bytes_no_copy_in(
                &v,
                cf::StringEncoding::UTF8,
                false,
                cf::Allocator::null(),
                None,
            )
            .unwrap();

            let _f = num;

            arr.show();
            arr
        };

        f.show()
    }
}
