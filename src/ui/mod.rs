pub mod tab_hardware;
pub mod tab_controller;
pub mod tab_cleaner;
pub mod tab_headset;
pub mod tab_security;
pub mod tab_idevice;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppTab {
    Hardware,
    Controller,
    Cleaner,
    Headset,
    Security,
    IDevice,
}

impl AppTab {
    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Hardware => "⚡ HARDWARE & POWER",
            Self::Controller => "🎮 CONTROLLER SPACE",
            Self::Cleaner => "🧹 SYSTEM CLEANER",
            Self::Headset => "🎧 HEADSET",
            Self::Security => "🛡️ SECURITY",
            Self::IDevice => "📱 iDEVICE",
        }
    }
}
