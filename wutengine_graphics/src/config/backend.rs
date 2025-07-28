//! Selectable graphics backends

use core::fmt::Display;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsBackend {
    pub dx12: bool,
    pub vulkan: bool,
    pub metal: bool,
    pub opengl: bool,
}

impl GraphicsBackend {
    pub const ALL: Self = GraphicsBackend {
        dx12: true,
        vulkan: true,
        metal: true,
        opengl: true,
    };

    pub const DX12: Self = GraphicsBackend {
        dx12: true,
        vulkan: false,
        metal: false,
        opengl: false,
    };

    pub const VULKAN: Self = GraphicsBackend {
        dx12: false,
        vulkan: true,
        metal: false,
        opengl: false,
    };

    pub const METAL: Self = GraphicsBackend {
        dx12: false,
        vulkan: false,
        metal: true,
        opengl: false,
    };

    pub const OPENGL: Self = GraphicsBackend {
        dx12: false,
        vulkan: false,
        metal: false,
        opengl: true,
    };

    pub const IN_BUILD: Self = GraphicsBackend {
        dx12: cfg!(feature = "dx12"),
        vulkan: cfg!(feature = "vulkan"),
        metal: cfg!(feature = "metal"),
        opengl: cfg!(feature = "opengl"),
    };
}

impl BitOr for GraphicsBackend {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            dx12: self.dx12 || rhs.dx12,
            vulkan: self.vulkan || rhs.vulkan,
            metal: self.metal || rhs.metal,
            opengl: self.opengl || rhs.opengl,
        }
    }
}

impl BitOrAssign for GraphicsBackend {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAnd for GraphicsBackend {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            dx12: self.dx12 && rhs.dx12,
            vulkan: self.vulkan && rhs.vulkan,
            metal: self.metal && rhs.metal,
            opengl: self.opengl && rhs.opengl,
        }
    }
}

impl BitAndAssign for GraphicsBackend {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl Not for GraphicsBackend {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            dx12: !self.dx12,
            vulkan: !self.vulkan,
            metal: !self.metal,
            opengl: !self.opengl,
        }
    }
}

impl Display for GraphicsBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut backends_strs = Vec::with_capacity(4);

        if self.dx12 {
            backends_strs.push("DirectX 12");
        }

        if self.vulkan {
            backends_strs.push("Vulkan");
        }

        if self.metal {
            backends_strs.push("Metal");
        }

        if self.opengl {
            backends_strs.push("OpenGL");
        }

        write!(f, "{}", backends_strs.join(", "))
    }
}

impl Default for GraphicsBackend {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                Self::VULKAN
            } else if #[cfg(target_os = "macos")] {
                Self::METAL
            } else if #[cfg(target_os = "linux")] {
                Self::VULKAN
            } else {
                Self::ALL
            }
        }
    }
}

impl From<GraphicsBackend> for wgpu::Backends {
    fn from(value: GraphicsBackend) -> Self {
        let mut backends = wgpu::Backends::empty();
        if value.dx12 {
            backends |= wgpu::Backends::DX12;
        }

        if value.vulkan {
            backends |= wgpu::Backends::VULKAN;
        }

        if value.metal {
            backends |= wgpu::Backends::METAL;
        }

        if value.opengl {
            backends |= wgpu::Backends::GL;
        }

        backends
    }
}
