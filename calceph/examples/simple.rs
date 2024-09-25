use calceph::{CalcephBin, PositionTarget, PositionUnit, TimeUnit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut peph = CalcephBin::new("calceph/examples/de440.bsp")?;

    peph.prefetch()?;

    dbg!(peph.get_constant("AU")?);

    dbg!(peph.compute_position_units(
        2442457.0,
        0.5,
        PositionTarget::MarsBarycenter,
        PositionTarget::SolarSystemBarycenter,
        PositionUnit::AstronomicalUnit,
        TimeUnit::Second
    )?);

    Ok(())
}
