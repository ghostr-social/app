const MAX_POINTS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
struct Point {
    at_ms: u64,
    probability: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct ProbabilityMass(Vec<Point>);

impl ProbabilityMass {
    fn point(at_ms: u64) -> Self {
        Self(vec![Point {
            at_ms,
            probability: 1.0,
        }])
    }

    fn normalize(mut points: Vec<Point>) -> Self {
        points.retain(|point| point.probability.is_finite() && point.probability > 0.0);
        points.sort_by_key(|point| point.at_ms);
        let mut combined: Vec<Point> = Vec::with_capacity(points.len());
        for point in points {
            match combined.last_mut().filter(|last| last.at_ms == point.at_ms) {
                Some(last) => last.probability += point.probability,
                None => combined.push(point),
            }
        }
        let total = combined.iter().map(|point| point.probability).sum::<f64>();
        combined
            .iter_mut()
            .for_each(|point| point.probability /= total.max(f64::MIN_POSITIVE));
        Self(compress(combined))
    }

    fn quantile(&self, probability: f64) -> u64 {
        let target = probability.clamp(0.0, 1.0);
        let mut cumulative = 0.0;
        for point in &self.0 {
            cumulative += point.probability;
            if cumulative >= target {
                return point.at_ms;
            }
        }
        self.0.last().map_or(0, |point| point.at_ms)
    }

    fn probability_by(&self, at_ms: u64) -> f64 {
        self.0
            .iter()
            .filter(|point| point.at_ms <= at_ms)
            .map(|point| point.probability)
            .sum::<f64>()
            .clamp(0.0, 1.0)
    }

    fn shifted(&self, offset_ms: u64) -> Self {
        Self::normalize(
            self.0
                .iter()
                .map(|point| Point {
                    at_ms: point.at_ms.saturating_add(offset_ms),
                    probability: point.probability,
                })
                .collect(),
        )
    }

    fn convolve(&self, other: &Self) -> Self {
        let mut points = Vec::with_capacity(self.0.len().saturating_mul(other.0.len()));
        for left in &self.0 {
            for right in &other.0 {
                points.push(Point {
                    at_ms: left.at_ms.saturating_add(right.at_ms),
                    probability: left.probability * right.probability,
                });
            }
        }
        Self::normalize(points)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WatchDistribution(ProbabilityMass);

impl WatchDistribution {
    pub(crate) fn from_survival(curve: &[(u64, f64)], horizon_ms: u64) -> Self {
        let mut points = Vec::with_capacity(curve.len() + 1);
        let mut previous = 1.0;
        for &(at_ms, raw_survival) in curve {
            let survival = raw_survival.clamp(0.0, previous);
            points.push(Point {
                at_ms: at_ms.saturating_sub(1),
                probability: previous - survival,
            });
            previous = survival;
        }
        points.push(Point {
            at_ms: horizon_ms,
            probability: previous,
        });
        Self(ProbabilityMass::normalize(points))
    }

    pub fn survival(&self, at_ms: u64) -> f64 {
        (1.0 - self.0.probability_by(at_ms.saturating_sub(1))).clamp(0.0, 1.0)
    }

    pub fn p50_ms(&self) -> u64 {
        self.0.quantile(0.50)
    }

    pub fn p95_ms(&self) -> u64 {
        self.0.quantile(0.95)
    }

    pub fn p99_ms(&self) -> u64 {
        self.0.quantile(0.99)
    }

    fn mass(&self) -> &ProbabilityMass {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeadlineDistribution(ProbabilityMass);

impl DeadlineDistribution {
    pub(crate) fn immediate() -> Self {
        Self(ProbabilityMass::point(0))
    }

    pub(crate) fn after_watch(&self, watch: &WatchDistribution) -> Self {
        Self(self.0.convolve(watch.mass()))
    }

    pub(crate) fn shifted(&self, offset_ms: u64) -> Self {
        Self(self.0.shifted(offset_ms))
    }

    pub fn p50_ms(&self) -> u64 {
        self.0.quantile(0.50)
    }

    pub fn p95_ms(&self) -> u64 {
        self.0.quantile(0.95)
    }

    pub fn p99_ms(&self) -> u64 {
        self.0.quantile(0.99)
    }

    pub fn probability_by(&self, at_ms: u64) -> f64 {
        self.0.probability_by(at_ms)
    }
}

fn compress(points: Vec<Point>) -> Vec<Point> {
    if points.len() <= MAX_POINTS {
        return points;
    }
    let mass = ProbabilityMass(points);
    let probability = 1.0 / MAX_POINTS as f64;
    let mut sampled = (0..MAX_POINTS)
        .map(|index| Point {
            at_ms: mass.quantile((index as f64 + 0.5) * probability),
            probability,
        })
        .collect::<Vec<_>>();
    sampled.sort_by_key(|point| point.at_ms);
    sampled
}
