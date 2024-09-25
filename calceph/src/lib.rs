use calceph_sys::*;
use std::ffi::{c_char, CStr};
use std::mem::MaybeUninit;
use std::sync::{LazyLock, Mutex};
use std::{ffi::CString, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bad filename")]
    BadFile,
    #[error("Lower level C Error: {0}")]
    LowerLevel(String),
}

// Is there a better way to do this?

static LAST_ERROR_STRING: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

fn update_last_error<T: AsRef<str>>(s: T) {
    let mut last_s = LAST_ERROR_STRING.lock().unwrap();
    *last_s = s.as_ref().to_owned()
}

#[no_mangle]
unsafe extern "C" fn error_handler_callback(str: *const c_char) {
    let s = CStr::from_ptr(str)
        .to_str()
        .expect("Error message wasn't UTF8");
    update_last_error(s);
}

fn get_last_error() -> String {
    LAST_ERROR_STRING.lock().unwrap().clone()
}

fn set_error_handler() {
    unsafe { calceph_seterrorhandler(3, Some(error_handler_callback)) }
}

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(not(unix))]
fn path_cstr(path: &Path) -> Result<CString, PdfiumError> {
    let path = path.to_str().ok_or(Error::BadFile)?;
    CString::new(path).map_err(|_| Error::BadFile)
}

#[cfg(unix)]
fn path_cstr(path: &Path) -> Result<CString, Error> {
    CString::new(path.as_os_str().as_bytes()).map_err(|_| Error::BadFile)
}

pub struct CalcephBin(*mut calcephbin);

impl Drop for CalcephBin {
    fn drop(&mut self) {
        unsafe {
            calceph_close(self.0);
        }
    }
}

#[cfg(feature = "threadsafe")]
unsafe impl Send for CalcephBin {}

#[derive(Debug)]
/// Values for `target` and `center` of [`CalephBin::compute`]
pub enum PositionTarget {
    MercuryBarycenter,
    VenusBarycenter,
    Earth,
    MarsBarycenter,
    JupiterBarycenter,
    SaturnBarycenter,
    UranusBarycenter,
    NeptuneBarycenter,
    PlutoBarycenter,
    Moon,
    Sun,
    SolarSystemBarycenter,
    EarthMoonBarycenter,
    Asteroid(i32),
}

impl From<PositionTarget> for i32 {
    fn from(value: PositionTarget) -> Self {
        match value {
            PositionTarget::MercuryBarycenter => 1,
            PositionTarget::VenusBarycenter => 2,
            PositionTarget::Earth => 3,
            PositionTarget::MarsBarycenter => 4,
            PositionTarget::JupiterBarycenter => 5,
            PositionTarget::SaturnBarycenter => 6,
            PositionTarget::UranusBarycenter => 7,
            PositionTarget::NeptuneBarycenter => 8,
            PositionTarget::PlutoBarycenter => 9,
            PositionTarget::Moon => 10,
            PositionTarget::Sun => 11,
            PositionTarget::SolarSystemBarycenter => 12,
            PositionTarget::EarthMoonBarycenter => 13,
            PositionTarget::Asteroid(x) => CALCEPH_ASTEROID as i32 + x,
        }
    }
}

#[derive(Debug)]
pub enum PositionUnit {
    AstronomicalUnit,
    Kilometer,
}

#[derive(Debug)]
pub enum TimeUnit {
    Day,
    Second,
}

impl From<PositionUnit> for i32 {
    fn from(value: PositionUnit) -> Self {
        (match value {
            PositionUnit::AstronomicalUnit => CALCEPH_UNIT_AU,
            PositionUnit::Kilometer => CALCEPH_UNIT_KM,
        }) as i32
    }
}

impl From<TimeUnit> for i32 {
    fn from(value: TimeUnit) -> Self {
        (match value {
            TimeUnit::Day => CALCEPH_UNIT_DAY,
            TimeUnit::Second => CALCEPH_UNIT_SEC,
        }) as i32
    }
}

#[derive(Debug)]
/// The timescale of the associated file
pub enum Timescale {
    TDB,
    TCB,
}

impl CalcephBin {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        set_error_handler();
        // Validate path
        let path = path_cstr(path.as_ref())?;
        let ptr = unsafe { calceph_open(path.as_ptr()) };
        // Try to construct the handler
        if ptr.is_null() {
            return Err(Error::LowerLevel(get_last_error()));
        }
        // If we want to be threadsafe, try to prefetch
        let mut this = Self(ptr);
        #[cfg(feature = "threadsafe")]
        this.prefetch()?;
        Ok(this)
    }

    pub fn prefetch(&mut self) -> Result<(), Error> {
        if unsafe { calceph_prefetch(self.0) } == 0 {
            Err(Error::LowerLevel(get_last_error()))
        } else {
            Ok(())
        }
    }

    pub fn get_timescale(&mut self) -> Timescale {
        let ret = unsafe { calceph_gettimescale(self.0) };
        match ret {
            1 => Timescale::TDB,
            2 => Timescale::TCB,
            _ => unreachable!(),
        }
    }

    pub fn compute_position_units(
        &mut self,
        ijd: f64,
        fdj: f64,
        target: PositionTarget,
        center: PositionTarget,
        pos_unit: PositionUnit,
        time_unit: TimeUnit,
    ) -> Result<[f64; 6], Error> {
        let mut pv = [0f64; 6];
        if unsafe {
            calceph_compute_unit(
                self.0,
                ijd,
                fdj,
                target.into(),
                center.into(),
                i32::from(pos_unit) + i32::from(time_unit),
                pv.as_mut_ptr(),
            )
        } == 0
        {
            Err(Error::LowerLevel(get_last_error()))
        } else {
            Ok(pv)
        }
    }

    pub fn get_constant(&mut self, name: &str) -> Result<f64, Error> {
        let c_str = CString::new(name).expect("Invalid C-string");
        let mut x = MaybeUninit::uninit();
        if unsafe { calceph_getconstant(self.0, c_str.as_ptr(), x.as_mut_ptr()) } == 0 {
            Err(Error::LowerLevel(get_last_error()))
        } else {
            Ok(unsafe { x.assume_init() })
        }
    }
}
