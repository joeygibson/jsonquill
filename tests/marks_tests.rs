use jsonquill::editor::marks::MarkSet;

#[test]
fn test_markset_creation() {
    let marks = MarkSet::new();
    assert_eq!(marks.get_mark('a'), None);
}

#[test]
fn test_set_and_get_mark() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0, 1, 2]);

    assert_eq!(marks.get_mark('a'), Some(&vec![0, 1, 2]));
    assert_eq!(marks.get_mark('b'), None);
}

#[test]
fn test_overwrite_mark() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0]);
    marks.set_mark('a', vec![1]);

    assert_eq!(marks.get_mark('a'), Some(&vec![1]));
}

#[test]
fn test_list_marks() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0]);
    marks.set_mark('c', vec![2]);

    let list = marks.list();
    assert_eq!(list.len(), 2);
    assert!(list.contains(&('a', &vec![0])));
    assert!(list.contains(&('c', &vec![2])));
}

#[test]
fn test_clear_marks() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0]);
    marks.set_mark('b', vec![1]);
    marks.clear();

    assert_eq!(marks.get_mark('a'), None);
    assert_eq!(marks.get_mark('b'), None);
}
