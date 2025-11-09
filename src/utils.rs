use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AllocationError {
    LengthMismatch,
    NegativeTotal,
    NegativeMinima,
    NaNOrNegativeWeight,
    Infeasible { required: i64, available: i64 },
    DownAdjustImpossible,
}

const EPSILON: f64 = 1e-12;

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationError::LengthMismatch => write!(f, "m and w must have the same length"),
            AllocationError::NegativeTotal => write!(f, "N must be non-negative"),
            AllocationError::NegativeMinima => write!(f, "all m_k must be non-negative"),
            AllocationError::NaNOrNegativeWeight => {
                write!(f, "weights must be finite and non-negative")
            }
            AllocationError::Infeasible {
                required,
                available,
            } => write!(f, "infeasible: sum(m) = {} > N = {}", required, available),
            AllocationError::DownAdjustImpossible => {
                write!(f, "could not decrease to meet N without violating minima")
            }
        }
    }
}
impl Error for AllocationError {}

/// Water-filling continuous solution:
///   x*_k = max(m_k, α * w_k) for some α s.t. sum x* = N
fn water_filling_continuous(
    total: i64,
    minimums: &[i64],
    weights: &[f64],
) -> Result<Vec<f64>, AllocationError> {
    let count = minimums.len() as usize;

    // Basic checks
    if minimums.len() != weights.len() {
        return Err(AllocationError::LengthMismatch);
    }
    if total < 0 {
        return Err(AllocationError::NegativeTotal);
    }
    if minimums.iter().any(|&mi| mi < 0) {
        return Err(AllocationError::NegativeMinima);
    }
    if weights.iter().any(|&wi| !wi.is_finite() || wi < 0.0) {
        return Err(AllocationError::NaNOrNegativeWeight);
    }

    let required: i64 = minimums.iter().sum();
    if required > total {
        return Err(AllocationError::Infeasible {
            required,
            available: total,
        });
    }

    // If all weights is zero, assume as same weights
    let all_zero_weight = weights.iter().all(|&wi| wi.abs() <= EPSILON);
    let weights = if all_zero_weight && total > 0 {
        vec![1.0f64; count]
    } else {
        weights.to_vec()
    };

    // clamped[i] == true means x_i = m_i
    let mut clamped = vec![false; count];
    let mut sum_m_l: f64 = 0.0;
    let mut sum_w_f: f64 = 0.0;

    // Initially clamp those with weight ~ 0
    for i in 0..count {
        if weights[i] <= 0.0 + EPSILON {
            clamped[i] = true;
            sum_m_l += minimums[i] as f64;
        } else {
            sum_w_f += weights[i];
        }
    }

    // Iteratively clamp violators where α w_i < m_i
    loop {
        if sum_w_f <= EPSILON {
            break;
        }
        let alpha = ((total as f64) - sum_m_l) / sum_w_f;

        let mut any = false;
        for i in 0..count {
            if !clamped[i] && alpha * weights[i] < (minimums[i] as f64) - EPSILON {
                clamped[i] = true;
                sum_m_l += minimums[i] as f64;
                sum_w_f -= weights[i];
                any = true;
            }
        }
        if !any {
            break;
        }
    }

    // Build continuous solution
    let mut x_star = vec![0.0f64; count];
    if sum_w_f <= EPSILON {
        // No free weights remain (or all zero): use minima
        for i in 0..count {
            x_star[i] = minimums[i] as f64;
        }
    } else {
        let alpha = ((total as f64) - sum_m_l) / sum_w_f;
        for i in 0..count {
            if clamped[i] {
                x_star[i] = minimums[i] as f64;
            } else {
                x_star[i] = alpha * weights[i];
            }
        }
    }

    // Adjust tiny numeric discrepancy (or distribute remainder deterministically if all weights ~0)
    let current_sum: f64 = x_star.iter().sum();
    let delta = (total as f64) - current_sum;
    if delta.abs() > 1e-7 {
        let free_count = clamped.iter().filter(|&&c| !c).count();
        if free_count > 0 {
            let add = delta / (free_count as f64);
            for i in 0..count {
                if !clamped[i] {
                    x_star[i] += add;
                }
            }
        } else {
            // No free set; spread across all
            let add = delta / (count as f64);
            for xi in &mut x_star {
                *xi += add;
            }
        }
    }

    Ok(x_star)
}

/// Integer rounding to sum N with lower bounds:
/// - base = max(m_i, floor(x*_i + 1e-12))
/// - give remaining units to largest fractional parts (ties broken deterministically)
fn round_to_sum_with_lowers(
    total: i64,
    x_star: &[f64],
    minimums: &[i64],
    weights: &[f64],
) -> Result<Vec<i64>, AllocationError> {
    let n = x_star.len();
    let eps_round = 1e-12;

    // Floor, respect minima
    let mut base: Vec<i64> = Vec::with_capacity(n);
    for i in 0..n {
        let flo = (x_star[i] + eps_round).floor() as i64;
        base.push(std::cmp::max(minimums[i], flo));
    }
    let bsum: i64 = base.iter().sum();
    let mut r = total - bsum;

    if r == 0 {
        return Ok(base);
    }

    let mut frac: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        let f = (x_star[i] - (base[i] as f64)).max(0.0);
        frac.push(f);
    }

    let mut x = base.clone();

    if r > 0 {
        // Give +1 to largest fractional parts; tiebreaker by higher weight, then smaller index
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&i, &j| {
            // Descending by frac, then descending by w, then ascending by index
            match frac[i]
                .partial_cmp(&frac[j])
                .unwrap_or(Ordering::Equal)
                .reverse()
            {
                Ordering::Equal => {
                    match weights[i]
                        .partial_cmp(&weights[j])
                        .unwrap_or(Ordering::Equal)
                        .reverse()
                    {
                        Ordering::Equal => i.cmp(&j), // smaller index first
                        other => other,
                    }
                }
                other => other,
            }
        });

        let mut k = 0;
        while r > 0 && k < n {
            let i = order[k];
            x[i] += 1;
            r -= 1;
            k += 1;
        }
        // If still remaining (all fracs 0 and n < r), cycle deterministically
        let mut i = 0usize;
        while r > 0 {
            x[i % n] += 1;
            r -= 1;
            i += 1;
        }
        Ok(x)
    } else {
        // Need to remove -r units; prefer smallest frac, then smaller weight, then smaller index
        let mut need = -r;
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&i, &j| {
            // Ascending by frac, then ascending by (-w), then ascending by index
            match frac[i].partial_cmp(&frac[j]).unwrap_or(Ordering::Equal) {
                Ordering::Equal => match (-weights[i])
                    .partial_cmp(&(-weights[j]))
                    .unwrap_or(Ordering::Equal)
                {
                    Ordering::Equal => i.cmp(&j),
                    other => other,
                },
                other => other,
            }
        });

        // Multiple passes if needed; stop if no progress
        while need > 0 {
            let mut progressed = false;
            for &i in &order {
                if need == 0 {
                    break;
                }
                if x[i] > minimums[i] {
                    x[i] -= 1;
                    need -= 1;
                    progressed = true;
                }
            }
            if !progressed {
                return Err(AllocationError::DownAdjustImpossible);
            }
        }
        Ok(x)
    }
}

pub fn allocate_weighted_with_minima(
    total: i64,
    minimums: &[i64],
    weights: &[f64],
) -> Result<Vec<i64>, AllocationError> {
    let x_star = water_filling_continuous(total, minimums, weights)?;
    let x_int = round_to_sum_with_lowers(total, &x_star, minimums, weights)?;
    Ok(x_int)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        {
            let total = 37;
            let minimums = vec![5, 0, 4, 0, 2];
            let weight = vec![1.0, 2.5, 0.5, 3.0, 1.0];

            let result = allocate_weighted_with_minima(total, &minimums, &weight).unwrap();
            assert_eq!(result.iter().sum::<i64>(), total);
            for i in 0..minimums.len() {
                assert!(result[i] >= minimums[i]);
            }

            // expected
            assert_eq!(result, vec![5, 11, 4, 13, 4]);
        }

        {
            let total = 10;
            let minimums = vec![9, 0];
            let weight = vec![1.0, 1.0];

            let result = allocate_weighted_with_minima(total, &minimums, &weight).unwrap();
            assert_eq!(result.iter().sum::<i64>(), total);
            for i in 0..minimums.len() {
                assert!(result[i] >= minimums[i]);
            }

            // expected
            assert_eq!(result, vec![9, 1]);
        }

        {
            let total = 10;
            let minimums = vec![10, 1];
            let weight = vec![1.0, 1.0];

            let err = allocate_weighted_with_minima(total, &minimums, &weight).unwrap_err();

            // expected
            match err {
                AllocationError::Infeasible { .. } => {}
                _ => panic!("expected infeasible"),
            }
        }
    }

    #[test]
    fn zero_weights_even_spread() {
        let total = 10;
        let minimums = vec![1, 2, 0, 0];
        let weights = vec![0.0, 0.0, 0.0, 0.0];

        let result = allocate_weighted_with_minima(total, &minimums, &weights).unwrap();
        assert_eq!(result.iter().sum::<i64>(), total);
        for i in 0..minimums.len() {
            assert!(result[i] >= minimums[i]);
        }

        // expected
        assert_eq!(result, vec![3, 3, 2, 2]);
    }
}
