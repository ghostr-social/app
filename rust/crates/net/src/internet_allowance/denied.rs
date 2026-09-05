/// Local admission failure. It supplies no evidence about an origin's health.
#[derive(Debug)]
pub struct InternetAdmissionDenied;

impl core::fmt::Display for InternetAdmissionDenied {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("Internet allowance admission denied")
    }
}

impl core::error::Error for InternetAdmissionDenied {}
