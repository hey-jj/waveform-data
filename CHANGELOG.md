# Changelog

## [0.1.1] - 2026-07-07

### Changed
- Resampling now returns invalid option errors for targets above `i32::MAX` instead of stalling or storing an invalid scale. (#19)

### Fixed
- Binary parsing now reports `DataLengthMismatch` when malformed length fields overflow the expected byte count. (#20)
- JSON parsing now reports `LengthMismatch` when malformed length or channel values overflow the expected sample count. (#21)

## [0.1.1] - 2026-07-07

### Changed
- Resampling now returns invalid option errors for targets above `i32::MAX` instead of stalling or storing an invalid scale. (#19)

### Fixed
- Binary parsing now reports `DataLengthMismatch` when malformed length fields overflow the expected byte count. (#20)
- JSON parsing now reports `LengthMismatch` when malformed length or channel values overflow the expected sample count. (#21)
