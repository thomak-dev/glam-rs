use std::fs::File;
use std::io::{Result, Write};

const E: [char; 4] = ['x', 'y', 'z', 'w']; // element name
const B: [&str; 4] = ["00", "01", "10", "11"]; // shuffle bits
const V: [&str; 4] = ["1", "2", "3", "4"]; // element value
const L: [&str; 4] = ["0", "1", "2", "3"]; // low index
const H: [&str; 4] = ["4", "5", "6", "7"]; // high index

// const VEC4: &str = "Vec4";
// const VEC3A: &str = "Vec3A";
// const VEC3: &str = "Vec3";
// const VEC2: &str = "Vec2";

fn write_swizzle_head(out: &mut impl Write) -> Result<()> {
    writeln!(out, "// Generated by swizzlegen. Do not edit.")?;
    Ok(())
}

fn write_loops_vec4<W, F4>(out: &mut W, size: usize, vec4fn: F4) -> Result<()>
where
    W: Write,
    F4: Fn(&mut W, usize, usize, usize, usize) -> Result<()>,
{
    for e0 in 0..size {
        for e1 in 0..size {
            for e2 in 0..size {
                for e3 in 0..size {
                    if size == 4 && e0 == 0 && e1 == 1 && e2 == 2 && e3 == 3 {
                        continue;
                    }
                    vec4fn(out, e0, e1, e2, e3)?;
                }
            }
        }
    }
    Ok(())
}

fn write_loops_vec3<W, F3>(out: &mut W, size: usize, vec3fn: F3) -> Result<()>
where
    W: Write,
    F3: Fn(&mut W, usize, usize, usize) -> Result<()>,
{
    for e0 in 0..size {
        for e1 in 0..size {
            for e2 in 0..size {
                if size == 3 && e0 == 0 && e1 == 1 && e2 == 2 {
                    continue;
                }
                vec3fn(out, e0, e1, e2)?;
            }
        }
    }
    Ok(())
}

fn write_loops_vec2<W, F2>(out: &mut W, size: usize, vec2fn: F2) -> Result<()>
where
    W: Write,
    F2: Fn(&mut W, usize, usize) -> Result<()>,
{
    for e0 in 0..size {
        for e1 in 0..size {
            if size == 2 && e0 == 0 && e1 == 1 {
                continue;
            }
            vec2fn(out, e0, e1)?;
        }
    }
    Ok(())
}

fn write_loops<W, F4, F3, F2>(
    out: &mut W,
    size: usize,
    vec4fn: F4,
    vec3fn: F3,
    vec2fn: F2,
) -> Result<()>
where
    W: Write,
    F4: Fn(&mut W, usize, usize, usize, usize) -> Result<()>,
    F3: Fn(&mut W, usize, usize, usize) -> Result<()>,
    F2: Fn(&mut W, usize, usize) -> Result<()>,
{
    write_loops_vec4(out, size, vec4fn)?;
    write_loops_vec3(out, size, vec3fn)?;
    write_loops_vec2(out, size, vec2fn)?;
    Ok(())
}

fn write_swizzle_trait(
    out: &mut impl Write,
    size: usize,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    let t = match size {
        4 => vec4t,
        3 => vec3t,
        2 => vec2t,
        _ => unreachable!(),
    };

    writeln!(out, r#"pub trait {}Swizzles: Sized + Copy + Clone {{"#, t)?;

    if size != 2 {
        writeln!(out, r#"    type Vec2;"#)?;
    }

    if size != 3 {
        writeln!(out, r#"    type Vec3;"#)?;
    }

    if size != 4 {
        writeln!(out, r#"    type Vec4;"#)?;
    }

    match size {
        4 => {
            writeln!(
                out,
                r#"
    #[inline]
    fn xyzw(self) -> Self {{
        self
    }}"#,
            )?;
        }
        3 => {
            writeln!(
                out,
                r#"
    #[inline]
    fn xyz(self) -> Self {{
        self
    }}"#,
            )?;
        }
        2 => {
            writeln!(
                out,
                r#"
    #[inline]
    fn xy(self) -> Self {{
        self
    }}"#,
            )?;
        }
        _ => unreachable!(),
    }

    write_loops(
        out,
        size,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    fn {}{}{}{}(self) -> {};"#,
                E[e0],
                E[e1],
                E[e2],
                E[e3],
                if size == 4 { "Self" } else { "Self::Vec4" }
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    fn {}{}{}(self) -> {};"#,
                E[e0],
                E[e1],
                E[e2],
                if size == 3 { "Self" } else { "Self::Vec3" }
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    fn {}{}(self) -> {};"#,
                E[e0],
                E[e1],
                if size == 2 { "Self" } else { "Self::Vec2" }
            )
        },
    )?;

    writeln!(
        out,
        r#"
}}"#
    )?;

    Ok(())
}

fn write_vec4_impl_scalar(
    out: &mut impl Write,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    const SIZE: usize = 4;

    write_swizzle_head(out)?;

    write!(
        out,
        r#"
use super::Vec4Swizzles;
use crate::{{{}, {}, {}}};
"#,
        vec2t, vec3t, vec4t,
    )?;

    write!(
        out,
        r#"
impl Vec4Swizzles for {} {{
    type Vec2 = {};
    type Vec3 = {};
"#,
        vec4t, vec2t, vec3t,
    )?;

    write_loops(
        out,
        SIZE,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}{}(self) -> {} {{
        {}::new(self.{}, self.{}, self.{}, self.{})
    }}"#,
                E[e0], E[e1], E[e2], E[e3], vec4t, vec4t, E[e0], E[e1], E[e2], E[e3],
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}(self) -> {} {{
        {}::new(self.{}, self.{}, self.{})
    }}"#,
                E[e0], E[e1], E[e2], vec3t, vec3t, E[e0], E[e1], E[e2]
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}(self) -> {} {{
        {}::new(self.{}, self.{})
    }}"#,
                E[e0], E[e1], vec2t, vec2t, E[e0], E[e1]
            )
        },
    )?;

    write!(out, "\n}}\n")?;

    Ok(())
}

fn write_vec4_impl_sse2(out: &mut impl Write) -> Result<()> {
    const SIZE: usize = 4;

    write_swizzle_head(out)?;

    write!(
        out,
        r#"
use super::Vec4Swizzles;
use crate::{{Vec2, Vec3, Vec4, XY, XYZ}};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
"#
    )?;

    write!(
        out,
        r#"
impl Vec4Swizzles for Vec4 {{
    type Vec2 = Vec2;
    type Vec3 = Vec3;
"#,
    )?;

    write_loops(
        out,
        SIZE,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}{}(self) -> Vec4 {{
        unsafe {{ Vec4(_mm_shuffle_ps(self.0, self.0, 0b{}_{}_{}_{})) }}
    }}"#,
                E[e0], E[e1], E[e2], E[e3], B[e3], B[e2], B[e1], B[e0],
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}(self) -> Vec3 {{
        unsafe {{ Vec3(XYZ::from(_mm_shuffle_ps(self.0, self.0, 0b00_{}_{}_{}))) }}
    }}"#,
                E[e0], E[e1], E[e2], B[e2], B[e1], B[e0],
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}(self) -> Vec2 {{
        unsafe {{ Vec2(XY::from(_mm_shuffle_ps(self.0, self.0, 0b00_00_{}_{}))) }}
    }}"#,
                E[e0], E[e1], B[e1], B[e0],
            )
        },
    )?;

    write!(out, "\n}}\n")?;

    Ok(())
}

fn write_vec3a_impl_sse2(out: &mut impl Write) -> Result<()> {
    const SIZE: usize = 3;

    write_swizzle_head(out)?;

    write!(
        out,
        r#"
use super::Vec3Swizzles;
use crate::{{Vec2, Vec3A, Vec4, XY}};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
"#
    )?;

    write!(
        out,
        r#"
impl Vec3Swizzles for Vec3A {{
    type Vec2 = Vec2;
    type Vec4 = Vec4;
"#
    )?;

    write_loops(
        out,
        SIZE,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}{}(self) -> Vec4 {{
        unsafe {{ Vec4(_mm_shuffle_ps(self.0, self.0, 0b{}_{}_{}_{})) }}
    }}"#,
                E[e0], E[e1], E[e2], E[e3], B[e3], B[e2], B[e1], B[e0],
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}(self) -> Self {{
        unsafe {{ Self(_mm_shuffle_ps(self.0, self.0, 0b00_{}_{}_{})) }}
    }}"#,
                E[e0], E[e1], E[e2], B[e2], B[e1], B[e0],
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}(self) -> Vec2 {{
        unsafe {{ Vec2(XY::from(_mm_shuffle_ps(self.0, self.0, 0b00_00_{}_{}))) }}
    }}"#,
                E[e0], E[e1], B[e1], B[e0],
            )
        },
    )?;

    write!(out, "\n}}\n")?;

    Ok(())
}

fn write_vec4_impl_wasm32(out: &mut impl Write) -> Result<()> {
    const SIZE: usize = 4;

    write_swizzle_head(out)?;

    write!(
        out,
        r#"
use super::Vec4Swizzles;
use crate::{{Vec2, Vec3, Vec4, XY, XYZ}};

use core::arch::wasm32::*;
"#
    )?;

    write!(
        out,
        r#"
impl Vec4Swizzles for Vec4 {{
    type Vec2 = Vec2;
    type Vec3 = Vec3;
"#,
    )?;

    write_loops(
        out,
        SIZE,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}{}(self) -> Vec4 {{
        Vec4(i32x4_shuffle::<{}, {}, {}, {}>(self.0, self.0))
    }}"#,
                E[e0], E[e1], E[e2], E[e3], L[e0], L[e1], H[e2], H[e3],
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}(self) -> Vec3 {{
        Vec3(XYZ::from(i32x4_shuffle::<{}, {}, {}, {}>(self.0, self.0)))
    }}"#,
                E[e0], E[e1], E[e2], L[e0], L[e1], H[e2], H[0],
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}(self) -> Vec2 {{
        Vec2(XY::from(i32x4_shuffle::<{}, {}, {}, {}>(self.0, self.0)))
    }}"#,
                E[e0], E[e1], L[e0], L[e1], H[0], H[0],
            )
        },
    )?;

    write!(out, "\n}}\n")?;

    Ok(())
}

fn write_vec3a_impl_wasm32(out: &mut impl Write) -> Result<()> {
    const SIZE: usize = 3;

    write_swizzle_head(out)?;

    write!(
        out,
        r#"
use super::Vec3Swizzles;
use crate::{{Vec2, Vec3A, Vec4, XY}};

use core::arch::wasm32::*;
"#
    )?;

    write!(
        out,
        r#"
impl Vec3Swizzles for Vec3A {{
    type Vec2 = Vec2;
    type Vec4 = Vec4;
"#
    )?;

    write_loops(
        out,
        SIZE,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}{}(self) -> Vec4 {{
        Vec4(i32x4_shuffle::<{}, {}, {}, {}>(self.0, self.0))
    }}"#,
                E[e0], E[e1], E[e2], E[e3], L[e0], L[e1], H[e2], H[e3],
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}(self) -> Self {{
        Self(i32x4_shuffle::<{}, {}, {}, {}>(self.0, self.0))
    }}"#,
                E[e0], E[e1], E[e2], L[e0], L[e1], H[e2], H[0],
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}(self) -> Vec2 {{
        Vec2(XY::from(i32x4_shuffle::<{}, {}, {}, {}>(self.0, self.0)))
    }}"#,
                E[e0], E[e1], L[e0], L[e1], H[0], H[0],
            )
        },
    )?;

    write!(out, "\n}}\n")?;

    Ok(())
}

fn write_vec3_impl_scalar(
    out: &mut impl Write,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    const SIZE: usize = 3;

    write_swizzle_head(out)?;

    write!(
        out,
        r#"
use super::Vec3Swizzles;
use crate::{{{}, {}, {}}};
"#,
        vec2t, vec3t, vec4t
    )?;

    write!(
        out,
        r#"
impl Vec3Swizzles for {} {{
    type Vec2 = {};
    type Vec4 = {};
"#,
        vec3t, vec2t, vec4t,
    )?;

    write_loops(
        out,
        SIZE,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}{}(self) -> {} {{
        {}::new(self.{}, self.{}, self.{}, self.{})
    }}"#,
                E[e0], E[e1], E[e2], E[e3], vec4t, vec4t, E[e0], E[e1], E[e2], E[e3],
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}(self) -> Self {{
        Self::new(self.{}, self.{}, self.{})
    }}"#,
                E[e0], E[e1], E[e2], E[e0], E[e1], E[e2]
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}(self) -> {} {{
        {}::new(self.{}, self.{})
    }}"#,
                E[e0], E[e1], vec2t, vec2t, E[e0], E[e1]
            )
        },
    )?;

    write!(out, "\n}}\n")?;

    Ok(())
}

fn write_vec2_impl_scalar(
    out: &mut impl Write,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    const SIZE: usize = 2;

    write_swizzle_head(out)?;

    write!(
        out,
        r#"
use super::Vec2Swizzles;
use crate::{{{}, {}, {}}};
"#,
        vec2t, vec3t, vec4t,
    )?;

    write!(
        out,
        r#"
impl Vec2Swizzles for {} {{
    type Vec3 = {};
    type Vec4 = {};
"#,
        vec2t, vec3t, vec4t,
    )?;

    write_loops(
        out,
        SIZE,
        |out, e0, e1, e2, e3| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}{}(self) -> {} {{
        {}::new(self.{}, self.{}, self.{}, self.{})
    }}"#,
                E[e0], E[e1], E[e2], E[e3], vec4t, vec4t, E[e0], E[e1], E[e2], E[e3],
            )
        },
        |out, e0, e1, e2| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}{}(self) -> {} {{
        {}::new(self.{}, self.{}, self.{})
    }}"#,
                E[e0], E[e1], E[e2], vec3t, vec3t, E[e0], E[e1], E[e2]
            )
        },
        |out, e0, e1| {
            write!(
                out,
                r#"
    #[inline]
    fn {}{}(self) -> Self {{
        Self::new(self.{}, self.{})
    }}"#,
                E[e0], E[e1], E[e0], E[e1]
            )
        },
    )?;

    write!(out, "\n}}\n")?;

    Ok(())
}

fn write_swizzle_traits() -> Result<()> {
    let mut out = File::create("../src/swizzles/vec_traits.rs")?;
    write_swizzle_head(&mut out)?;
    writeln!(
        out,
        r#"/** Swizzle methods for 2-dimensional vector types. */"#
    )?;
    write_swizzle_trait(&mut out, 2, "Vec4", "Vec3", "Vec2")?;
    writeln!(
        out,
        r#"/** Swizzle methods for 3-dimensional vector types. */"#
    )?;
    write_swizzle_trait(&mut out, 3, "Vec4", "Vec3", "Vec2")?;
    writeln!(
        out,
        r#"/** Swizzle methods for 4-dimensional vector types. */"#
    )?;
    write_swizzle_trait(&mut out, 4, "Vec4", "Vec3", "Vec2")?;

    Ok(())
}

fn write_swizzle_impls_f32() -> Result<()> {
    let mut out = File::create("../src/swizzles/vec4_impl_scalar.rs")?;
    write_vec4_impl_scalar(&mut out, "Vec4", "Vec3", "Vec2")?;

    let mut out = File::create("../src/swizzles/vec3_impl_scalar.rs")?;
    write_vec3_impl_scalar(&mut out, "Vec4", "Vec3", "Vec2")?;

    let mut out = File::create("../src/swizzles/vec2_impl_scalar.rs")?;
    write_vec2_impl_scalar(&mut out, "Vec4", "Vec3", "Vec2")?;

    let mut out = File::create("../src/swizzles/vec3a_impl_scalar.rs")?;
    write_vec3_impl_scalar(&mut out, "Vec4", "Vec3A", "Vec2")?;

    let mut out = File::create("../src/swizzles/vec4_impl_sse2.rs")?;
    write_vec4_impl_sse2(&mut out)?;

    let mut out = File::create("../src/swizzles/vec3a_impl_sse2.rs")?;
    write_vec3a_impl_sse2(&mut out)?;

    let mut out = File::create("../src/swizzles/vec4_impl_wasm32.rs")?;
    write_vec4_impl_wasm32(&mut out)?;

    let mut out = File::create("../src/swizzles/vec3a_impl_wasm32.rs")?;
    write_vec3a_impl_wasm32(&mut out)?;
    Ok(())
}

fn write_swizzle_impls_f64() -> Result<()> {
    let mut out = File::create("../src/swizzles/dvec4_impl_scalar.rs")?;
    write_vec4_impl_scalar(&mut out, "DVec4", "DVec3", "DVec2")?;

    let mut out = File::create("../src/swizzles/dvec3_impl_scalar.rs")?;
    write_vec3_impl_scalar(&mut out, "DVec4", "DVec3", "DVec2")?;

    let mut out = File::create("../src/swizzles/dvec2_impl_scalar.rs")?;
    write_vec2_impl_scalar(&mut out, "DVec4", "DVec3", "DVec2")?;

    Ok(())
}

fn write_swizzle_impls_i32() -> Result<()> {
    let mut out = File::create("../src/swizzles/ivec4_impl_scalar.rs")?;
    write_vec4_impl_scalar(&mut out, "IVec4", "IVec3", "IVec2")?;

    let mut out = File::create("../src/swizzles/ivec3_impl_scalar.rs")?;
    write_vec3_impl_scalar(&mut out, "IVec4", "IVec3", "IVec2")?;

    let mut out = File::create("../src/swizzles/ivec2_impl_scalar.rs")?;
    write_vec2_impl_scalar(&mut out, "IVec4", "IVec3", "IVec2")?;

    Ok(())
}

fn write_swizzle_impls_u32() -> Result<()> {
    let mut out = File::create("../src/swizzles/uvec4_impl_scalar.rs")?;
    write_vec4_impl_scalar(&mut out, "UVec4", "UVec3", "UVec2")?;

    let mut out = File::create("../src/swizzles/uvec3_impl_scalar.rs")?;
    write_vec3_impl_scalar(&mut out, "UVec4", "UVec3", "UVec2")?;

    let mut out = File::create("../src/swizzles/uvec2_impl_scalar.rs")?;
    write_vec2_impl_scalar(&mut out, "UVec4", "UVec3", "UVec2")?;

    Ok(())
}

fn write_test_vec4(
    out: &mut impl Write,
    t: &str,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    const SIZE: usize = 4;

    write!(
        out,
        r#"
glam_test!(test_{}_swizzles, {{
    let v = {}(1_{}, 2_{}, 3_{}, 4_{});
"#,
        vec4t, vec4t, t, t, t, t,
    )?;

    writeln!(out, "    assert_eq!(v, v.xyzw());")?;

    write_test_loops(out, SIZE, t, vec4t, vec3t, vec2t)?;

    writeln!(out, "}});")?;

    Ok(())
}

fn write_test_vec3(
    out: &mut impl Write,
    t: &str,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    const SIZE: usize = 3;

    write!(
        out,
        r#"
glam_test!(test_{}_swizzles, {{
    let v = {}(1_{}, 2_{}, 3_{});
"#,
        vec3t, vec3t, t, t, t,
    )?;

    writeln!(out, "    assert_eq!(v, v.xyz());")?;

    write_test_loops(out, SIZE, t, vec4t, vec3t, vec2t)?;

    writeln!(out, "}});")?;

    Ok(())
}

fn write_test_vec2(
    out: &mut impl Write,
    t: &str,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    const SIZE: usize = 2;

    write!(
        out,
        r#"
glam_test!(test_{}_swizzles, {{
    let v = {}(1_{}, 2_{});
"#,
        vec2t, vec2t, t, t,
    )?;

    writeln!(out, "    assert_eq!(v, v.xy());")?;

    write_test_loops(out, SIZE, t, vec4t, vec3t, vec2t)?;

    writeln!(out, "}});")?;

    Ok(())
}

fn write_test_loops(
    out: &mut impl Write,
    size: usize,
    t: &str,
    vec4t: &str,
    vec3t: &str,
    vec2t: &str,
) -> Result<()> {
    write_loops_vec4(out, size, |out, e0, e1, e2, e3| {
        writeln!(
            out,
            "    assert_eq!(v.{}{}{}{}(), {}({}_{}, {}_{}, {}_{}, {}_{}));",
            E[e0], E[e1], E[e2], E[e3], vec4t, V[e0], t, V[e1], t, V[e2], t, V[e3], t
        )
    })?;
    write_loops_vec3(out, size, |out, e0, e1, e2| {
        writeln!(
            out,
            "    assert_eq!(v.{}{}{}(), {}({}_{}, {}_{}, {}_{}));",
            E[e0], E[e1], E[e2], vec3t, V[e0], t, V[e1], t, V[e2], t,
        )
    })?;
    write_loops_vec2(out, size, |out, e0, e1| {
        writeln!(
            out,
            "    assert_eq!(v.{}{}(), {}({}_{}, {}_{}));",
            E[e0], E[e1], vec2t, V[e0], t, V[e1], t,
        )
    })?;
    Ok(())
}

fn write_swizzle_tests_preamble(filename: &str) -> Result<impl Write> {
    let mut out = File::create(filename)?;
    write_swizzle_head(&mut out)?;
    writeln!(
        &mut out,
        r#"#[macro_use]
mod support;
use glam::*;
"#
    )?;
    Ok(out)
}

fn write_swizzle_tests() -> Result<()> {
    {
        let mut out = write_swizzle_tests_preamble("../tests/swizzles_f32.rs")?;
        write_test_vec4(&mut out, "f32", "vec4", "vec3", "vec2")?;
        write_test_vec3(&mut out, "f32", "vec4", "vec3a", "vec2")?;
        write_test_vec3(&mut out, "f32", "vec4", "vec3", "vec2")?;
        write_test_vec2(&mut out, "f32", "vec4", "vec3", "vec2")?;
    }

    // split f64 swizzle tests up so they don't exceed some wasm code size
    {
        let mut out = write_swizzle_tests_preamble("../tests/swizzles_f64.rs")?;
        write_test_vec4(&mut out, "f64", "dvec4", "dvec3", "dvec2")?;
        write_test_vec3(&mut out, "f64", "dvec4", "dvec3", "dvec2")?;
        write_test_vec2(&mut out, "f64", "dvec4", "dvec3", "dvec2")?;
    }

    {
        let mut out = write_swizzle_tests_preamble("../tests/swizzles_i32.rs")?;
        write_test_vec4(&mut out, "i32", "ivec4", "ivec3", "ivec2")?;
        write_test_vec3(&mut out, "i32", "ivec4", "ivec3", "ivec2")?;
        write_test_vec2(&mut out, "i32", "ivec4", "ivec3", "ivec2")?;
    }

    {
        let mut out = write_swizzle_tests_preamble("../tests/swizzles_u32.rs")?;
        write_test_vec4(&mut out, "u32", "uvec4", "uvec3", "uvec2")?;
        write_test_vec3(&mut out, "u32", "uvec4", "uvec3", "uvec2")?;
        write_test_vec2(&mut out, "u32", "uvec4", "uvec3", "uvec2")?;
    }

    Ok(())
}

fn main() -> Result<()> {
    write_swizzle_traits()?;
    write_swizzle_impls_f32()?;
    write_swizzle_impls_f64()?;
    write_swizzle_impls_i32()?;
    write_swizzle_impls_u32()?;
    write_swizzle_tests()?;
    Ok(())
}
