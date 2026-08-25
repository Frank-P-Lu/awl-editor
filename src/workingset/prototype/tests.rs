use super::*;

#[test]
fn move_candidate_has_one_explicit_action_pair_before_the_real_folder_rows() {
    let folders = vec!["journal/".to_string(), "research/".to_string()];
    assert_eq!(
        prototype_move_rows(&folders),
        ["Move here", "New folder…", "journal/", "research/"]
    );
    assert_eq!(folders, ["journal/", "research/"], "source rows stay inert");
}
