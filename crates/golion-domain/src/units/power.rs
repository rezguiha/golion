use derive_more::{Add, From, Into};
use jiff::SignedDuration;
// region: Power Units

#[derive(PartialEq, From, Add, Into, Debug, Clone, Copy)]
pub struct KiloWatt(pub f64);

#[derive(PartialEq, From, Add, Into, Debug, Clone, Copy)]
pub struct MegaWatt(pub f64);

impl From<KiloWatt> for MegaWatt {
    fn from(v: KiloWatt) -> Self {
        let v_float: f64 = v.into();
        MegaWatt(v_float / 1000.0)
    }
}
// endregion: Power Units

// region: Energy units
#[derive(PartialEq, From, Add, Into, Debug, Clone, Copy)]
pub struct KiloWattHour(pub f64);

#[derive(PartialEq, From, Add, Into, Debug, Clone, Copy)]
pub struct MegaWattHour(pub f64);

impl From<KiloWattHour> for MegaWattHour {
    fn from(v: KiloWattHour) -> MegaWattHour {
        MegaWattHour(v.0 / 1000.0)
    }
}

// endregion: Energy units

// region: Power-Energy Conversion
impl KiloWatt {
    pub fn to_kwh(&self, duration: SignedDuration) -> KiloWattHour {
        KiloWattHour(self.0 * duration.as_secs_f64() / 3600.0)
    }
    pub fn to_mwh(&self, duration: SignedDuration) -> MegaWattHour {
        self.to_kwh(duration).into()
    }
}
impl MegaWatt {
    pub fn to_kwh(&self, duration: SignedDuration) -> KiloWattHour {
        let value_kw: KiloWatt = KiloWatt(self.0 * 1000.0);
        value_kw.to_kwh(duration)
    }
    pub fn to_mwh(&self, duration: SignedDuration) -> MegaWattHour {
        MegaWattHour(self.0 * duration.as_secs_f64() / 3600.0)
    }
}

// endregion: Power-Energy Conversion
