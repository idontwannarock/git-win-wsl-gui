use crate::log::Oid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRow {
    pub column: usize,
    pub passthrough: Vec<usize>,
    pub merges_in: Vec<usize>,
    pub parents_out: Vec<usize>,
    pub width: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LaneLayout {
    lanes: Vec<Option<Oid>>,
}

impl LaneLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_row(&mut self, id: &Oid, parents: &[Oid]) -> GraphRow {
        let width_before = self.lanes.len();
        let merges_in: Vec<usize> = self.lane_indexes(|lane| lane.as_ref() == Some(id));
        let passthrough: Vec<usize> = self
            .lane_indexes(Option::is_some)
            .into_iter()
            .filter(|index| !merges_in.contains(index))
            .collect();

        let column = match merges_in.first() {
            Some(&column) => column,
            None => self.free_slot(),
        };
        for &index in &merges_in {
            self.lanes[index] = None;
        }

        let mut parents_out = Vec::with_capacity(parents.len());
        for (position, parent) in parents.iter().enumerate() {
            let existing = self.lanes.iter().position(|lane| lane.as_ref() == Some(parent));
            let slot = match existing {
                Some(slot) => slot,
                None => {
                    let slot = if position == 0 { column } else { self.free_slot() };
                    self.lanes[slot] = Some(parent.clone());
                    slot
                }
            };
            parents_out.push(slot);
        }

        while self.lanes.last() == Some(&None) {
            self.lanes.pop();
        }
        GraphRow {
            column,
            passthrough,
            merges_in,
            parents_out,
            width: width_before.max(self.lanes.len()).max(column + 1),
        }
    }

    fn lane_indexes(&self, keep: impl Fn(&Option<Oid>) -> bool) -> Vec<usize> {
        self.lanes
            .iter()
            .enumerate()
            .filter(|(_, lane)| keep(lane))
            .map(|(index, _)| index)
            .collect()
    }

    fn free_slot(&mut self) -> usize {
        match self.lanes.iter().position(Option::is_none) {
            Some(slot) => slot,
            None => {
                self.lanes.push(None);
                self.lanes.len() - 1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oid(number: u32) -> Oid {
        Oid::parse(&format!("{number:040x}")).unwrap()
    }

    fn feed(lanes: &mut LaneLayout, history: &[(u32, &[u32])]) -> Vec<GraphRow> {
        history
            .iter()
            .map(|(id, parents)| {
                let parents: Vec<Oid> = parents.iter().map(|&parent| oid(parent)).collect();
                lanes.next_row(&oid(*id), &parents)
            })
            .collect()
    }

    fn layout(history: &[(u32, &[u32])]) -> Vec<GraphRow> {
        feed(&mut LaneLayout::new(), history)
    }

    fn row(column: usize, passthrough: &[usize], merges_in: &[usize], parents_out: &[usize], width: usize) -> GraphRow {
        GraphRow {
            column,
            passthrough: passthrough.to_vec(),
            merges_in: merges_in.to_vec(),
            parents_out: parents_out.to_vec(),
            width,
        }
    }

    #[test]
    fn a_linear_history_stays_in_one_column() {
        let rows = layout(&[(3, &[2]), (2, &[1]), (1, &[])]);
        assert_eq!(
            rows,
            [
                row(0, &[], &[], &[0], 1),
                row(0, &[], &[0], &[0], 1),
                row(0, &[], &[0], &[], 1),
            ]
        );
    }

    #[test]
    fn a_fork_and_merge_uses_two_columns_and_rejoins() {
        let rows = layout(&[(4, &[2, 3]), (2, &[1]), (3, &[1]), (1, &[])]);
        assert_eq!(
            rows,
            [
                row(0, &[], &[], &[0, 1], 2),
                row(0, &[1], &[0], &[0], 2),
                row(1, &[0], &[1], &[0], 2),
                row(0, &[], &[0], &[], 1),
            ]
        );
    }

    #[test]
    fn an_octopus_merge_opens_one_column_per_parent() {
        let rows = layout(&[(4, &[1, 2, 3]), (1, &[]), (2, &[]), (3, &[])]);
        assert_eq!(
            rows,
            [
                row(0, &[], &[], &[0, 1, 2], 3),
                row(0, &[1, 2], &[0], &[], 3),
                row(1, &[2], &[1], &[], 3),
                row(2, &[], &[2], &[], 3),
            ]
        );
    }

    #[test]
    fn two_tips_on_one_parent_join_without_a_new_lane_after_the_tip() {
        let rows = layout(&[(2, &[1]), (3, &[1]), (1, &[])]);
        assert_eq!(
            rows,
            [
                row(0, &[], &[], &[0], 1),
                row(1, &[0], &[], &[0], 2),
                row(0, &[], &[0], &[], 1),
            ]
        );
    }

    #[test]
    fn a_column_freed_by_a_root_commit_is_reused() {
        let rows = layout(&[(2, &[1]), (1, &[]), (3, &[])]);
        assert_eq!(rows[2].column, 0);
        assert_eq!(rows[2].width, 1);
    }

    #[test]
    fn a_new_tip_takes_the_first_free_column_between_busy_ones() {
        let rows = layout(&[(10, &[1, 2, 3]), (1, &[]), (11, &[4]), (2, &[]), (3, &[]), (4, &[])]);
        assert_eq!(rows[2].column, 0);
        assert_eq!(rows[2].passthrough, [1, 2]);
    }

    #[test]
    fn rows_depend_only_on_the_state_so_a_clone_resumes_the_same_way() {
        let history: &[(u32, &[u32])] = &[(4, &[2, 3]), (2, &[1]), (3, &[1]), (1, &[])];
        let all = layout(history);

        let mut first_half = LaneLayout::new();
        let head = feed(&mut first_half, &history[..2]);
        let mut resumed = first_half.clone();
        let tail = feed(&mut resumed, &history[2..]);

        assert_eq!([head, tail].concat(), all);
    }
}
