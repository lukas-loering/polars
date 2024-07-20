use super::*;

macro_rules! impl_unit_setter {
    ($fn_name:ident($field:ident)) => {
        #[doc = concat!("Set the ", stringify!($field))]
        pub fn $fn_name(mut self, n: Expr) -> Self {
            self.$field = n.into();
            self
        }
    };
}

/// Arguments used by `datetime` in order to produce an [`Expr`] of Datetime
///
/// Construct a [`DatetimeArgs`] with `DatetimeArgs::new(y, m, d)`. This will set the other time units to `lit(0)`. You
/// can then set the other fields with the `with_*` methods, or use `with_hms` to set `hour`, `minute`, and `second` all
/// at once.
///
/// # Examples
/// ```
/// use polars_plan::prelude::*;
/// // construct a DatetimeArgs set to July 20, 1969 at 20:17
/// let args = DatetimeArgs::new(lit(1969), lit(7), lit(20)).with_hms(lit(20), lit(17), lit(0));
/// // or
/// let args = DatetimeArgs::new(lit(1969), lit(7), lit(20)).with_hour(lit(20)).with_minute(lit(17));
///
/// // construct a DatetimeArgs using existing columns
/// let args = DatetimeArgs::new(lit(2023), col("month"), col("day"));
/// ```
#[derive(Debug, Clone)]
pub struct DatetimeArgs {
    pub year: Expr,
    pub month: Expr,
    pub day: Expr,
    pub hour: Expr,
    pub minute: Expr,
    pub second: Expr,
    pub microsecond: Expr,
    pub time_unit: TimeUnit,
    pub time_zone: Option<TimeZone>,
    /// DST (Daylight Saving Time) may cause some local times to occur more than once on the same day.
    /// `ambiguous` is a  [`DataType::String`] expression that defines how to handle ambiguous datetimes:
    ///
    /// - `raise`: (default) raise an error
    /// - `earliest`: use the earliest datetime
    /// - `latest`: use the latest datetime
    /// - `null`: set to null
    pub ambiguous: Expr,
}

impl Default for DatetimeArgs {
    fn default() -> Self {
        Self {
            year: lit(1970),
            month: lit(1),
            day: lit(1),
            hour: lit(0),
            minute: lit(0),
            second: lit(0),
            microsecond: lit(0),
            time_unit: TimeUnit::Microseconds,
            time_zone: None,
            ambiguous: lit(String::from("raise")),
        }
    }
}

impl DatetimeArgs {
    /// Construct a new `DatetimeArgs` set to `year`, `month`, `day`
    ///
    /// Other fields default to `lit(0)`. Use the `with_*` methods to set them.
    pub fn new(year: Expr, month: Expr, day: Expr) -> Self {
        Self {
            year,
            month,
            day,
            ..Default::default()
        }
    }

    /// Set `hour`, `minute`, and `second`
    ///
    /// Equivalent to
    /// ```ignore
    /// self.with_hour(hour)
    ///     .with_minute(minute)
    ///     .with_second(second)
    /// ```
    pub fn with_hms(self, hour: Expr, minute: Expr, second: Expr) -> Self {
        Self {
            hour,
            minute,
            second,
            ..self
        }
    }

    impl_unit_setter!(with_year(year));
    impl_unit_setter!(with_month(month));
    impl_unit_setter!(with_day(day));
    impl_unit_setter!(with_hour(hour));
    impl_unit_setter!(with_minute(minute));
    impl_unit_setter!(with_second(second));
    impl_unit_setter!(with_microsecond(microsecond));

    pub fn with_time_unit(self, time_unit: TimeUnit) -> Self {
        Self { time_unit, ..self }
    }
    #[cfg(feature = "timezones")]
    pub fn with_time_zone(self, time_zone: Option<TimeZone>) -> Self {
        Self { time_zone, ..self }
    }
    /// # Ambiguous Datetimes
    /// DST (Daylight Saving Time) may cause some local times to occur more than once on the same day.
    /// `ambiguous` is a  [`DataType::String`] expression that defines how to handle ambiguous datetimes:
    ///
    /// - `raise`: (default) raise an error
    /// - `earliest`: use the earliest datetime
    /// - `latest`: use the latest datetime
    /// - `null`: set to null
    #[cfg(feature = "timezones")]
    pub fn with_ambiguous(self, ambiguous: Expr) -> Self {
        Self { ambiguous, ..self }
    }
}

/// Construct a column of `Datetime` from the provided [`DatetimeArgs`].
#[cfg(feature = "temporal")]
pub fn datetime(args: DatetimeArgs) -> Expr {
    let year = args.year;
    let month = args.month;
    let day = args.day;
    let hour = args.hour;
    let minute = args.minute;
    let second = args.second;
    let microsecond = args.microsecond;
    let time_unit = args.time_unit;
    let time_zone = args.time_zone;
    let ambiguous = args.ambiguous;

    let input = vec![
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
        ambiguous,
    ];

    Expr::Function {
        input,
        function: FunctionExpr::TemporalExpr(TemporalFunction::DatetimeFunction {
            time_unit,
            time_zone,
        }),
        options: FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            flags: FunctionFlags::default()
                | FunctionFlags::INPUT_WILDCARD_EXPANSION
                | FunctionFlags::ALLOW_RENAME,
            fmt_str: "datetime",
            ..Default::default()
        },
    }
}

/// Arguments used by `duration` in order to produce an [`Expr`] of [`Duration`]
///
/// To construct a [`DurationArgs`], use struct literal syntax with `..Default::default()` to leave unspecified fields at
/// their default value of `lit(0)`, as demonstrated below.
///
/// ```
/// # use polars_plan::prelude::*;
/// let args = DurationArgs {
///     days: lit(5),
///     hours: col("num_hours"),
///     minutes: col("num_minutes"),
///     ..Default::default()  // other fields are lit(0)
/// };
/// ```
/// If you prefer builder syntax, `with_*` methods are also available.
/// ```
/// # use polars_plan::prelude::*;
/// let args = DurationArgs::new().with_weeks(lit(42)).with_hours(lit(84));
/// ```
#[derive(Debug, Clone)]
pub struct DurationArgs {
    pub weeks: Expr,
    pub days: Expr,
    pub hours: Expr,
    pub minutes: Expr,
    pub seconds: Expr,
    pub milliseconds: Expr,
    pub microseconds: Expr,
    pub nanoseconds: Expr,
    pub time_unit: TimeUnit,
}

impl Default for DurationArgs {
    fn default() -> Self {
        Self {
            weeks: lit(0),
            days: lit(0),
            hours: lit(0),
            minutes: lit(0),
            seconds: lit(0),
            milliseconds: lit(0),
            microseconds: lit(0),
            nanoseconds: lit(0),
            time_unit: TimeUnit::Microseconds,
        }
    }
}

impl DurationArgs {
    /// Create a new [`DurationArgs`] with all fields set to `lit(0)`. Use the `with_*` methods to set the fields.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set `hours`, `minutes`, and `seconds`
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// self.with_hours(hours)
    ///     .with_minutes(minutes)
    ///     .with_seconds(seconds)
    /// ```
    pub fn with_hms(self, hours: Expr, minutes: Expr, seconds: Expr) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            ..self
        }
    }

    /// Set `milliseconds`, `microseconds`, and `nanoseconds`
    ///
    /// Equivalent to
    /// ```ignore
    /// self.with_milliseconds(milliseconds)
    ///     .with_microseconds(microseconds)
    ///     .with_nanoseconds(nanoseconds)
    /// ```
    pub fn with_fractional_seconds(
        self,
        milliseconds: Expr,
        microseconds: Expr,
        nanoseconds: Expr,
    ) -> Self {
        Self {
            milliseconds,
            microseconds,
            nanoseconds,
            ..self
        }
    }

    impl_unit_setter!(with_weeks(weeks));
    impl_unit_setter!(with_days(days));
    impl_unit_setter!(with_hours(hours));
    impl_unit_setter!(with_minutes(minutes));
    impl_unit_setter!(with_seconds(seconds));
    impl_unit_setter!(with_milliseconds(milliseconds));
    impl_unit_setter!(with_microseconds(microseconds));
    impl_unit_setter!(with_nanoseconds(nanoseconds));
}

/// Construct a column of [`Duration`] from the provided [`DurationArgs`]
#[cfg(feature = "temporal")]
pub fn duration(args: DurationArgs) -> Expr {
    Expr::Function {
        input: vec![
            args.weeks,
            args.days,
            args.hours,
            args.minutes,
            args.seconds,
            args.milliseconds,
            args.microseconds,
            args.nanoseconds,
        ],
        function: FunctionExpr::TemporalExpr(TemporalFunction::Duration(args.time_unit)),
        options: FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            flags: FunctionFlags::default() | FunctionFlags::INPUT_WILDCARD_EXPANSION,
            ..Default::default()
        },
    }
}
