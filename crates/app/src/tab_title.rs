fn segments(path: &str) -> Vec<&str> {
    path.split(['\\', '/']).filter(|segment| !segment.is_empty()).collect()
}

pub fn titles(paths: &[String]) -> Vec<String> {
    let names: Vec<&str> = paths
        .iter()
        .map(|path| segments(path).last().copied().unwrap_or_default())
        .collect();
    paths
        .iter()
        .zip(&names)
        .map(|(path, name)| {
            let shared = names.iter().filter(|other| *other == name).count() > 1;
            let parent = segments(path).iter().rev().nth(1).copied();
            match parent {
                Some(parent) if shared => format!("{name} · {parent}"),
                _ => (*name).to_owned(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles_of(paths: &[&str]) -> Vec<String> {
        titles(&paths.iter().map(|path| path.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn a_single_tab_shows_its_folder_name() {
        assert_eq!(titles_of(&[r"C:\code\repo"]), ["repo"]);
    }

    #[test]
    fn different_names_stay_plain() {
        assert_eq!(titles_of(&[r"C:\code\one", r"C:\code\two"]), ["one", "two"]);
    }

    #[test]
    fn equal_names_both_add_the_parent_folder() {
        assert_eq!(
            titles_of(&[r"C:\a\app", r"\\wsl$\Ubuntu\home\me\app"]),
            ["app · a", "app · me"]
        );
    }

    #[test]
    fn a_unique_name_next_to_a_pair_stays_plain() {
        assert_eq!(
            titles_of(&[r"C:\a\app", r"C:\b\app", r"C:\code\other"]),
            ["app · a", "app · b", "other"]
        );
    }

    #[test]
    fn a_trailing_separator_does_not_change_the_name() {
        assert_eq!(titles_of(&[r"C:\code\repo\"]), ["repo"]);
    }

    #[test]
    fn a_forward_slash_path_works() {
        assert_eq!(titles_of(&["C:/code/repo"]), ["repo"]);
    }

    #[test]
    fn a_drive_root_uses_the_drive_as_its_name() {
        assert_eq!(titles_of(&[r"C:\"]), ["C:"]);
    }

    #[test]
    fn equal_names_without_a_parent_stay_plain() {
        assert_eq!(titles_of(&[r"C:\", r"C:\"]), ["C:", "C:"]);
    }

    #[test]
    fn an_empty_list_gives_no_titles() {
        assert!(titles(&[]).is_empty());
    }
}
