//! Viewport selection is independent of the DOM so ranking/scroll edge cases can be tested.
use std::collections::{BTreeMap, HashMap};

pub const TRANSITION_MS: f64 = 5_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub top: f64,
    pub height: f64,
    pub row_height: f64,
}

impl Viewport {
    pub fn range(self, count: usize, offset: usize) -> std::ops::Range<usize> {
        let first =
            ((self.top - self.height) / self.row_height).floor() as isize + offset as isize - 1;
        let end =
            ((self.top + self.height * 2.0) / self.row_height).ceil() as isize + offset as isize;
        first.clamp(0, count as isize) as usize..end.clamp(0, count as isize) as usize
    }

    fn visible(self, y: f64) -> bool {
        y + self.row_height >= self.top && y <= self.top + self.height
    }

    fn boundary(self, y: f64) -> f64 {
        y.clamp(self.top - self.row_height, self.top + self.height)
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub initial_y: f64,
    pub y: f64,
    pub expires: Option<f64>,
}

#[derive(Default)]
pub struct WindowRows {
    pub rows: BTreeMap<String, Row>,
}

impl WindowRows {
    pub fn update(
        &mut self,
        order: &[String],
        positions: &HashMap<String, usize>,
        previous: &HashMap<String, usize>,
        viewport: Viewport,
        offsets: (usize, usize),
        ranking_changed: bool,
        now: f64,
    ) {
        let (offset, previous_offset) = offsets;
        let y = |p: usize, offset: usize| (p as f64 - offset as f64) * viewport.row_height;
        let range = viewport.range(order.len(), offset);
        self.rows.retain(|login, row| {
            let Some(&position) = positions.get(login) else {
                return false;
            };
            let wanted = range.contains(&(position - 1));
            if wanted {
                row.y = y(position, offset);
                row.expires = None;
                return true;
            }
            if ranking_changed && row.expires.is_none() && viewport.visible(row.y) {
                row.y = viewport.boundary(y(position, offset));
                row.expires = Some(now + TRANSITION_MS);
            }
            if ranking_changed && row.expires.is_some() {
                row.y = viewport.boundary(y(position, offset));
            }
            row.expires.is_some_and(|deadline| now < deadline)
        });
        for index in range {
            let login = &order[index];
            self.rows.entry(login.clone()).or_insert_with(|| {
                let target = y(index + 1, offset);
                let initial_y = if ranking_changed {
                    previous
                        .get(login)
                        .map(|p| viewport.boundary(y(*p, previous_offset)))
                        .unwrap_or(target)
                } else {
                    target
                };
                Row {
                    initial_y,
                    y: target,
                    expires: None,
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn positions(order: &[String]) -> HashMap<String, usize> {
        order
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i + 1))
            .collect()
    }
    #[test]
    fn bounds_scroll_center_and_empty() {
        let v = Viewport {
            top: 0.0,
            height: 480.0,
            row_height: 48.0,
        };
        assert_eq!(v.range(5000, 0), 0..20);
        assert_eq!(v.range(5000, 4991), 4980..5000);
        assert_eq!(v.range(0, 0), 0..0);
        assert_eq!(Viewport { top: 239520.0, ..v }.range(5000, 0), 4979..5000);
    }
    #[test]
    fn distant_entry_exit_expiry_and_scroll_are_bounded() {
        let mut order = (0..5000).map(|i| i.to_string()).collect::<Vec<_>>();
        let previous = positions(&order);
        let v = Viewport {
            top: 0.0,
            height: 480.0,
            row_height: 48.0,
        };
        let mut window = WindowRows::default();
        window.update(&order, &previous, &previous, v, (0, 0), false, 0.0);
        order.swap(0, 4999);
        let next = positions(&order);
        window.update(&order, &next, &previous, v, (0, 0), true, 1.0);
        assert_eq!(window.rows["4999"].initial_y, 480.0);
        assert_eq!(window.rows["4999"].y, 48.0);
        assert_eq!(window.rows["0"].expires, Some(5001.0));
        window.update(&order, &next, &next, v, (0, 0), false, 5002.0);
        assert!(!window.rows.contains_key("0"));
        for top in (0..239000).step_by(480) {
            window.update(
                &order,
                &next,
                &next,
                Viewport {
                    top: top as f64,
                    ..v
                },
                (0, 0),
                false,
                6000.0,
            );
            assert!(window.rows.len() <= 31);
        }
    }
}
