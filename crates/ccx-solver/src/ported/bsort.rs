//! Rust port of `bsort.f`.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BSortBounds {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub dmax: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BSortError {
    InvalidDmax,
    InvalidBounds,
    MissingX { index: usize },
    MissingY { index: usize },
    MissingBin { index: usize },
}

pub fn bsort(
    list: &mut [usize],
    bin: &mut [i32],
    x: &[f64],
    y: &[f64],
    bounds: BSortBounds,
) -> Result<(), BSortError> {
    if list.is_empty() {
        return Ok(());
    }
    if !bounds.dmax.is_finite() || bounds.dmax == 0.0 {
        return Err(BSortError::InvalidDmax);
    }

    let ndiv = (list.len() as f64).powf(0.25).round() as i32;
    let x_span = (bounds.xmax - bounds.xmin) * 1.01 / bounds.dmax;
    let y_span = (bounds.ymax - bounds.ymin) * 1.01 / bounds.dmax;
    if !x_span.is_finite() || !y_span.is_finite() || x_span == 0.0 || y_span == 0.0 {
        return Err(BSortError::InvalidBounds);
    }

    let factx = f64::from(ndiv) / x_span;
    let facty = f64::from(ndiv) / y_span;

    for &p in list.iter() {
        let xp = *x.get(p).ok_or(BSortError::MissingX { index: p })?;
        let yp = *y.get(p).ok_or(BSortError::MissingY { index: p })?;
        let target = bin.get_mut(p).ok_or(BSortError::MissingBin { index: p })?;
        let i = (yp * facty) as i32;
        let j = (xp * factx) as i32;
        *target = if i % 2 == 0 {
            i * ndiv + j + 1
        } else {
            (i + 1) * ndiv - j
        };
    }

    list.sort_unstable_by_key(|&p| bin[p]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BSortBounds, BSortError, bsort};

    #[test]
    fn computes_bins_and_sorts_index_list() {
        let x = vec![0.1, 1.2, 2.8, 0.3];
        let y = vec![0.2, 1.8, 0.7, 2.2];
        let mut list = vec![0usize, 1, 2, 3];
        let mut bin = vec![0i32; 4];

        bsort(
            &mut list,
            &mut bin,
            &x,
            &y,
            BSortBounds {
                xmin: 0.0,
                xmax: 3.0,
                ymin: 0.0,
                ymax: 3.0,
                dmax: 1.0,
            },
        )
        .expect("bsort should succeed");

        assert!(list.windows(2).all(|w| bin[w[0]] <= bin[w[1]]));
    }

    #[test]
    fn rejects_invalid_dmax() {
        let x = vec![0.0];
        let y = vec![0.0];
        let mut list = vec![0usize];
        let mut bin = vec![0i32; 1];

        let err = bsort(
            &mut list,
            &mut bin,
            &x,
            &y,
            BSortBounds {
                xmin: 0.0,
                xmax: 1.0,
                ymin: 0.0,
                ymax: 1.0,
                dmax: 0.0,
            },
        )
        .expect_err("dmax = 0 should fail");

        assert_eq!(err, BSortError::InvalidDmax);
    }

    #[test]
    fn rejects_invalid_bounds() {
        let x = vec![0.0];
        let y = vec![0.0];
        let mut list = vec![0usize];
        let mut bin = vec![0i32; 1];

        let err = bsort(
            &mut list,
            &mut bin,
            &x,
            &y,
            BSortBounds {
                xmin: 1.0,
                xmax: 1.0,
                ymin: 0.0,
                ymax: 1.0,
                dmax: 1.0,
            },
        )
        .expect_err("zero x span should fail");

        assert_eq!(err, BSortError::InvalidBounds);
    }

    #[test]
    fn reports_missing_coordinate_or_bin_indices() {
        let mut list = vec![1usize];
        let mut bin = vec![0i32; 1];
        let x = vec![0.1];
        let y = vec![0.2];

        let err = bsort(
            &mut list,
            &mut bin,
            &x,
            &y,
            BSortBounds {
                xmin: 0.0,
                xmax: 1.0,
                ymin: 0.0,
                ymax: 1.0,
                dmax: 1.0,
            },
        )
        .expect_err("index 1 is out of bounds");

        assert_eq!(err, BSortError::MissingX { index: 1 });
    }
}
