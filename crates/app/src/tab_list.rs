pub struct TabList<T> {
    entries: Vec<T>,
    active: Option<usize>,
}

impl<T> TabList<T> {
    pub fn new() -> Self {
        Self { entries: Vec::new(), active: None }
    }

    pub fn entries(&self) -> &[T] {
        &self.entries
    }

    pub fn active(&self) -> Option<usize> {
        self.active
    }

    pub fn entry_mut(&mut self, index: usize) -> Option<&mut T> {
        self.entries.get_mut(index)
    }

    pub fn open(&mut self, entry: T) -> usize {
        self.entries.push(entry);
        let index = self.entries.len() - 1;
        self.active = Some(index);
        index
    }

    pub fn activate(&mut self, index: usize) -> bool {
        let exists = index < self.entries.len();
        if exists {
            self.active = Some(index);
        }
        exists
    }

    pub fn close(&mut self, index: usize) -> Option<T> {
        if index >= self.entries.len() {
            return None;
        }
        let entry = self.entries.remove(index);
        self.active = match self.active {
            _ if self.entries.is_empty() => None,
            Some(active) if active > index => Some(active - 1),
            Some(active) if active == index => Some(index.min(self.entries.len() - 1)),
            other => other,
        };
        Some(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list_of(names: &[&'static str]) -> TabList<&'static str> {
        let mut list = TabList::new();
        for name in names {
            list.open(*name);
        }
        list
    }

    #[test]
    fn a_new_list_is_empty_and_has_no_active_tab() {
        let list: TabList<&str> = TabList::new();
        assert!(list.entries().is_empty());
        assert_eq!(list.active(), None);
    }

    #[test]
    fn open_adds_at_the_end_and_activates_it() {
        let mut list = list_of(&["a"]);
        assert_eq!(list.open("b"), 1);
        assert_eq!(list.entries(), ["a", "b"]);
        assert_eq!(list.active(), Some(1));
    }

    #[test]
    fn activate_moves_the_active_tab() {
        let mut list = list_of(&["a", "b", "c"]);
        assert!(list.activate(0));
        assert_eq!(list.active(), Some(0));
    }

    #[test]
    fn activate_ignores_an_index_that_does_not_exist() {
        let mut list = list_of(&["a", "b"]);
        assert!(!list.activate(5));
        assert_eq!(list.active(), Some(1));
    }

    #[test]
    fn closing_the_active_tab_activates_its_right_neighbor() {
        let mut list = list_of(&["a", "b", "c"]);
        list.activate(1);
        assert_eq!(list.close(1), Some("b"));
        assert_eq!(list.entries(), ["a", "c"]);
        assert_eq!(list.entries()[list.active().unwrap()], "c");
    }

    #[test]
    fn closing_the_active_last_tab_activates_its_left_neighbor() {
        let mut list = list_of(&["a", "b"]);
        assert_eq!(list.close(1), Some("b"));
        assert_eq!(list.entries()[list.active().unwrap()], "a");
    }

    #[test]
    fn closing_the_only_tab_leaves_no_active_tab() {
        let mut list = list_of(&["a"]);
        assert_eq!(list.close(0), Some("a"));
        assert!(list.entries().is_empty());
        assert_eq!(list.active(), None);
    }

    #[test]
    fn closing_a_tab_left_of_the_active_one_keeps_the_same_tab_active() {
        let mut list = list_of(&["a", "b", "c"]);
        list.close(0);
        assert_eq!(list.entries()[list.active().unwrap()], "c");
    }

    #[test]
    fn closing_a_tab_right_of_the_active_one_keeps_the_same_tab_active() {
        let mut list = list_of(&["a", "b", "c"]);
        list.activate(0);
        list.close(2);
        assert_eq!(list.entries()[list.active().unwrap()], "a");
    }

    #[test]
    fn closing_an_index_that_does_not_exist_changes_nothing() {
        let mut list = list_of(&["a", "b"]);
        assert_eq!(list.close(9), None);
        assert_eq!(list.entries(), ["a", "b"]);
        assert_eq!(list.active(), Some(1));
    }
}
