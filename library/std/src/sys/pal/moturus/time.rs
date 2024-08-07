use crate::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Instant(moto_runtime::time::Instant);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SystemTime(moto_runtime::time::SystemTime);

pub const UNIX_EPOCH: SystemTime = SystemTime(moto_runtime::time::UNIX_EPOCH);

impl Instant {
    pub fn now() -> Instant {
        Self(moto_runtime::time::Instant::now())
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        self.0.checked_sub_instant(&other.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant(self.0.checked_add_duration(other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant(self.0.checked_sub_duration(other)?))
    }
}

impl SystemTime {
    pub fn now() -> SystemTime {
        Self(moto_runtime::time::SystemTime::now())
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.0.sub_time(&other.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_add_duration(other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        self.0.checked_sub_duration(other).map(|f| Self(f))
    }

    pub fn as_unix_ts(&self) -> u64 {
        self.0.as_unix_ts()
    }

    pub fn from_unix_ts(ts: u64) -> Self {
        Self(moto_runtime::time::SystemTime::from_unix_ts(ts))
    }
}
