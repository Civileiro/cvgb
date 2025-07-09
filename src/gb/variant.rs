use std::cell::Cell;

use enum_assoc::Assoc;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Assoc, Clone)]
#[func(pub fn is_color_variant(&self) -> bool)]
pub enum GameBoyVariant {
    #[assoc(is_color_variant = false)]
    DMG0,
    #[assoc(is_color_variant = false)]
    DMG,
    #[assoc(is_color_variant = true)]
    CGB0 { dmg_compatibility: Cell<bool> },
    #[assoc(is_color_variant = true)]
    CGB { dmg_compatibility: Cell<bool> },
}

impl GameBoyVariant {
    pub fn is_dmg_compatible(&self) -> bool {
        match self {
            GameBoyVariant::DMG0 => true,
            GameBoyVariant::DMG => true,
            GameBoyVariant::CGB0 { dmg_compatibility } => dmg_compatibility.get(),
            GameBoyVariant::CGB { dmg_compatibility } => dmg_compatibility.get(),
        }
    }
    pub fn set_dmg_compatibility(&self) {
        match self {
            GameBoyVariant::CGB0 { dmg_compatibility } => dmg_compatibility.set(true),
            GameBoyVariant::CGB { dmg_compatibility } => dmg_compatibility.set(true),
            _ => (),
        }
    }
}
